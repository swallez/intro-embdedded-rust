
// Sample code at https://github.com/ivmarkov/rust-esp32-std-demo/blob/main/src/main.rs

use esp_idf_sys as _;

use anyhow::bail;
use anyhow::Result;

use esp_idf_hal::adc;
use esp_idf_hal::peripherals::Peripherals;

use log::{info, warn};
use build_time::build_time_local;

use std::{ sync::atomic::*, sync::Arc, time::*};
use esp_idf_hal::peripheral::Peripheral;

struct WifiCredentials {
    ssid: &'static str,
    pass: &'static str,
}

// Define the wifi access point credentials in a separate file
//const WIFI: WifiCredentials = include!("../../config/wifi.example.txt");
const WIFI: WifiCredentials = include!("../../config/wifi-home.txt");
//const WIFI: WifiCredentials = include!("../../config/wifi-phone.txt");


fn main() -> Result<()> {
    esp_idf_sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    //esp_idf_svc::log::EspLogger{}.set_target_level("*", log::LevelFilter::Warn);
    esp_idf_svc::log::EspLogger::initialize_default();
    //log::set_max_level(log::LevelFilter::Info);

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    // Setup data shared between tasks
    let light_value = Arc::new(AtomicU16::new(0));

    // Initialize Wifi
    let mut _wifi = init_net_and_wifi(peripherals.modem)?;

    // Create an http server
    let _httpd = httpd(light_value.clone())?;

    //---------------------------------

    let mut adc1 = adc::AdcDriver::new(peripherals.adc1, &adc::AdcConfig::new().calibration(true))?;

    let d0 = pins.gpio0;
    let mut adc_pin: adc::AdcChannelDriver<_, adc::Atten11dB<_>> = adc::AdcChannelDriver::new(d0)?;

    let stop = false;

    while !stop {

        let value = adc1.read(&mut adc_pin)?;

        light_value.store(value, Ordering::SeqCst);

        std::thread::sleep(Duration::from_millis(500));
    };

    Ok(())
}

fn init_net_and_wifi(
    modem: impl Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
) -> Result<(esp_idf_svc::wifi::EspWifi<'static>, esp_idf_svc::mdns::EspMdns)>{

    // See https://docs.espressif.com/projects/esp-idf/en/latest/esp32c3/api-guides/wifi.html

    use esp_idf_svc::wifi::*;
    use embedded_svc::wifi::*;
    use esp_idf_svc::netif::*;
    use std::net::Ipv4Addr;

    let nvs = esp_idf_svc::nvs::EspDefaultNvsPartition::take()?;
    let sysloop = esp_idf_svc::eventloop::EspSystemEventLoop::take()?;
    let mut wifi = EspWifi::new(modem, sysloop.clone(), Some(nvs))?;

    // Use station (i.e. client) mode. We could also use mixed client/AP mode to create an
    // AP on which users can configure the real AP name and credentials.
    wifi.set_configuration(&Configuration::Client(
        ClientConfiguration {
            ssid: WIFI.ssid.into(),
            password: WIFI.pass.into(),
            ..Default::default()
        }
    ))?;

    let wifi_wait = WifiWait::new(&sysloop)?;
    let wait_delay = Duration::from_secs(20);

    // Start the WIFI subsystem
    info!("Starting wifi");
    wifi.start()?;

    if !wifi_wait.wait_with_timeout(wait_delay, || wifi.is_started().unwrap()) {
        bail!("Wifi did not start");
    }

    // Since we're in station (client) mode, connect to the AP and wait for
    // the DHCP server to give us an IP address.
    info!("Connecting wifi & waiting for DHCP lease");
    wifi.connect()?;

    let netif_wait = EspNetifWait::new::<EspNetif>(wifi.sta_netif(), &sysloop)?;

    if !netif_wait.wait_with_timeout(wait_delay, ||
        // Connected to AP
        wifi.is_connected().unwrap() &&
        // Got an IP address
        wifi.sta_netif().get_ip_info().unwrap().ip != Ipv4Addr::new(0, 0, 0, 0)
    ) {
        bail!("Wifi did not connect or did not receive a DHCP lease");
    }

    let ip_info = wifi.sta_netif().get_ip_info()?;
    warn!("Wifi connected. Go to http://{}", &ip_info.ip);

    // Declare ourselves on mDNS and declare our http service on DNS-SD.
    let mut mdns = esp_idf_svc::mdns::EspMdns::take()?;
    mdns.set_hostname("hello-esp32")?;
    mdns.add_service(
        Some("Hello ESP32"),
        "_http", "_tcp", 80,
        &[("build-time", build_time_local!())]
    )?;

    warn!("On a machine with zeroconf, go to http://hello-esp32.local");

    // These objects must be owned forever, or else the services will be shutdown.
    Ok((wifi, mdns))
}

fn httpd(
    light: Arc<AtomicU16>
) -> Result<esp_idf_svc::http::server::EspHttpServer> {

    use embedded_svc::http::Method;
    use esp_idf_svc::http::server::Configuration;
    use esp_idf_svc::http::server::EspHttpServer;
    use embedded_svc::io::Write;

    let mut server = EspHttpServer::new(&Configuration::default())?;

    server
        .fn_handler("/", Method::Get, |req| {
            req.into_response(
                    200, Some("Ok"),
                    &[("Content-Type", "text/html")])?
                .write_all(
                    r#"
                        <p>Hello from Rust on ESP32!</p>
                        <p><a href="/light">Light sensor</a></p>
                    "#.as_bytes())
                ?;
            Ok(())
        })?

        .fn_handler("/light", Method::Get, move |req| {

            let mut resp = req.into_response(
                200, Some("Ok"),
                &[("Content-Type", "text/html")])?;

            write!(resp,
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
