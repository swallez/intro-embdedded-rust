use esp_idf_sys as _;

use anyhow::anyhow;

use esp_idf_hal::i2c;
use esp_idf_hal::peripherals::Peripherals;

use anyhow::Result;

use esp_idf_hal::adc;

use embedded_graphics::mono_font::{ascii::FONT_10X20, MonoTextStyle};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::*;

use std::{ sync::atomic::*, sync::Arc};
use std::time::Duration;

use ssd1306::prelude::*;

fn main() -> Result<()> {
    esp_idf_sys::link_patches(); // Will disappear once ESP-IDF 4.4

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    // Initialize i2c bus for the display
    let i2c_driver = i2c::I2cDriver::new(
        peripherals.i2c0,
        pins.gpio2, // sda
        pins.gpio3, // sci
        &i2c::I2cConfig::default()
    )?;

    // Initialize ssd1306 display on i2c
    let display_itf = ssd1306::I2CDisplayInterface::new(i2c_driver);

    let mut display = ssd1306::Ssd1306::new(
        display_itf,
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    ).into_buffered_graphics_mode();

    display.init()
        .map_err(display_error)?;

    draw_text(&mut display, "Hello, world")?;

    display.flush()
        .map_err(display_error)?;

    std::thread::sleep(Duration::from_secs(5));
    //delay::FreeRtos::delay_ms(5000);

    // Setup data shared between tasks
    let light_value = Arc::new(AtomicU16::new(0));
    //---------------------------------
    // Configure ADC

    let mut adc1 = adc::AdcDriver::new(peripherals.adc1, &adc::AdcConfig::new().calibration(true))?;

    let d0 = pins.gpio0;
    let mut adc_pin: adc::AdcChannelDriver<_, adc::Atten11dB<_>> = adc::AdcChannelDriver::new(d0)?;

    //---------------------------------
    // Loop to display the light measurement

    let stop = false;
    while !stop {

        let value = adc1.read(&mut adc_pin)?;

        light_value.store(value, Ordering::SeqCst);

        draw_text(&mut display, &format!("Light: {}", value))?;
        display.flush()
            .map_err(display_error)?;

        std::thread::sleep(Duration::from_millis(500));
    };

    Ok(())
}

fn draw_text<D>(display: &mut D, text: &str) -> Result<()>
    where
        D: DrawTarget<Color = BinaryColor> + Dimensions,
{
    display.clear(BinaryColor::Off)
        .map_err(|_| anyhow!("Display error"))?;

    let text = Text::new(
        text,
        Point::new(5, 40),
        MonoTextStyle::new(&FONT_10X20, BinaryColor::On),
    );

    text.draw(display)
        .map_err(|_| anyhow!("Display error"))?;

    Ok(())
}

fn display_error(err: display_interface::DisplayError) -> anyhow::Error {
    anyhow!("{:?}", err)
}

// cargo build --release --bin hello-display
// cargo run --release --bin hello-display
// objdump -h target/riscv32imc-esp-espidf/release/hello-display
