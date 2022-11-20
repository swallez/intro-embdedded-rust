use esp_idf_sys as _;

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;

use esp_idf_hal::adc;
use esp_idf_hal::delay;
use esp_idf_hal::i2c;
use esp_idf_hal::peripherals::Peripherals;

use embedded_hal::adc::OneShot;
use embedded_hal::blocking::delay::DelayMs;

use log::warn;

use embedded_graphics::mono_font::{ascii::FONT_10X20, MonoTextStyle};
use embedded_graphics::pixelcolor::*;
use embedded_graphics::prelude::*;
use embedded_graphics::text::*;

use ssd1306::mode::DisplayConfig;
use ssd1306::prelude::*;

use std::{ sync::atomic::*, sync::Arc, time::*};
use embedded_svc::http::server::registry::Registry;
use esp_idf_svc::http::server::EspHttpResponse;

struct WifiCredentials {
    ssid: &'static str,
    pass: &'static str,
}

// Define the wifi access point credentials in a separate file
const WIFI: WifiCredentials = include!("../../config/wifi-example.txt");
//const WIFI: WifiCredentials = include!("../../config/wifi-home.txt");
//const WIFI: WifiCredentials = include!("../../config/wifi-phone.txt");


fn main() -> Result<()> {
    esp_idf_sys::link_patches(); // Will disappear once ESP-IDF 4.4

    println!("Hello, world!");

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    log::set_max_level(log::LevelFilter::Warn);

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    // Initialize i2c bus for the display
    let i2c_master = i2c::Master::new(
        peripherals.i2c0,
        i2c::MasterPins { sda: pins.gpio2, scl: pins.gpio3 },
        i2c::config::MasterConfig::default(),
    )?;

    // Initialize a ssd1306 on i2c bus
    let display_itf = ssd1306::I2CDisplayInterface::new(i2c_master);

    let mut display = ssd1306::Ssd1306::new(
        display_itf,
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    ).into_buffered_graphics_mode();

    display.init()
        .map_err(|e| anyhow::anyhow!("Display error: {:?}", e))?;

    // Setup data shared between tasks
    let light_value = Arc::new(AtomicU16::new(0));

    // Initialize Wifi
    let _wifi = init_net_and_wifi()?;

    // Create an http server
    let _httpd = httpd(light_value.clone())?;

    //---------------------------------

    let mut d0 = pins.gpio0.into_analog_atten_11db()?;

    let mut powered_adc1 = adc::PoweredAdc::new(
        peripherals.adc1,
        adc::config::Config::new().calibration(true),
    )?;

    let stop = false;

    while !stop {

        let value = powered_adc1.read(&mut d0).unwrap();

        light_value.store(value, Ordering::SeqCst);

        draw_text(&mut display, &format!("{}", value)).map_err(|e| anyhow::anyhow!("Display error: {:?}", e))?;
        display
            .flush()
            .map_err(|e| anyhow::anyhow!("Display error: {:?}", e))?;

        delay::Ets.delay_ms(300u32);
    };

    Ok(())
}

fn draw_text<D>(display: &mut D, text: &str) -> Result<()>
    where
        D: DrawTarget<Color = BinaryColor> + Dimensions,
{
    display.clear(BinaryColor::Off)
        .map_err(|_| anyhow!("Display error"))?;

    Text::new(
        text,
        Point::new(10, 40),
        MonoTextStyle::new(&FONT_10X20, BinaryColor::On),
    )
        .draw(display)
        .map_err(|_| anyhow!("Display error"))?;

    Ok(())
}

fn init_net_and_wifi() -> Result<Box<esp_idf_svc::wifi::EspWifi>> {

    let netif_stack = Arc::new(esp_idf_svc::netif::EspNetifStack::new()?);
    let sys_loop_stack = Arc::new(esp_idf_svc::sysloop::EspSysLoopStack::new()?);
    let default_nvs = Arc::new(esp_idf_svc::nvs::EspDefaultNvs::new()?);

    let wifi = configure_wifi(netif_stack, sys_loop_stack, default_nvs)?;

    Ok(wifi)
}

fn configure_wifi(
    netif_stack: Arc<esp_idf_svc::netif::EspNetifStack>,
    sys_loop_stack: Arc<esp_idf_svc::sysloop::EspSysLoopStack>,
    default_nvs: Arc<esp_idf_svc::nvs::EspDefaultNvs>,
) -> Result<Box<esp_idf_svc::wifi::EspWifi>> {

    use esp_idf_svc::wifi::EspWifi;
    use embedded_svc::wifi::*;

    let mut wifi = Box::new(EspWifi::new(netif_stack, sys_loop_stack, default_nvs)?);

    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == WIFI.ssid);

    let channel = if let Some(ours) = ours {
        warn!("Found access point {} on channel {}",WIFI.ssid, ours.channel);
        Some(ours.channel)
    } else {
        warn!("Access point {} not found during scanning", WIFI.ssid);
        None
    };

    wifi.set_configuration(&Configuration::Mixed(
        ClientConfiguration {
            ssid: WIFI.ssid.into(),
            password: WIFI.pass.into(),
            channel,
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: "aptest".into(),
            channel: channel.unwrap_or(1),
            ..Default::default()
        },
    ))?;

    wifi.wait_status_with_timeout(Duration::from_secs(20), |status| !status.is_transitional())
        .map_err(|e| anyhow::anyhow!("Unexpected Wifi status: {:?}", e))?;

    let status = wifi.get_status();

    if let Status(
        ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(ip_settings))),
        ApStatus::Started(ApIpStatus::Done),
    ) = status
    {
        warn!("Wifi connected. Go to http://{}", &ip_settings.ip);
    } else {
        bail!("Unexpected Wifi status: {:?}", status);
    }

    Ok(wifi)
}

fn httpd(
    light: Arc<AtomicU16>
) -> Result<esp_idf_svc::http::server::EspHttpServer> {
    use embedded_svc::http::server::Response;
    use embedded_svc::http::server::SendHeaders;
    use embedded_svc::http::SendStatus;
    use esp_idf_svc::http::server::Configuration;
    use esp_idf_svc::http::server::EspHttpServer;
    use embedded_svc::io::Write;

    let mut server = EspHttpServer::new(&Configuration::default())?;
    server
        .handle_get("/", |_req, resp| {
            resp.status(200)
                .header("Content-Type", "text/html")
                .send_str(r#"
                    <p>Hello from Rust on ESP32!</p>
                    <p><a href="/light">Light sensor</a></p>
                "#)?;
            Ok(())
        })?

        .handle_get("/light", move |_req, resp: EspHttpResponse| {

            let mut w = resp
                .header("Content-Type", "text/html")
                .status(200)
                .into_writer()?;

            write!(w,
                r#"
                    <html>
                    <head><meta http-equiv="refresh" content="1"></head>
                    <body>Light sensor: {}</body>
                    </html>
                "#,
                light.load(Ordering::SeqCst)
            )?;

            Ok(())
        })?;

    Ok(server)
}

// cargo build --release --bin hello-http
// cargo run --release --bin hello-http
// objdump -h target/riscv32imc-esp-espidf/release/hello-http
