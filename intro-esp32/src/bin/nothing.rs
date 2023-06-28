use esp_idf_sys as _;

use anyhow::anyhow;
use anyhow::Result;
use log::info;

// Do nothing (except clearing the display).
// Used to set the microcontroller in a "blank state" before starting the demos.
fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    clear_display()?;

    info!("Doing nothing, as requested.");
    Ok(())
}

fn clear_display() -> Result<()> {
    use esp_idf_hal::i2c;
    use esp_idf_hal::peripherals::Peripherals;
    use ssd1306::prelude::*;

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

    display.init().map_err(display_error)?;

    display.clear();
    display.flush().map_err(display_error)?;

    Ok(())
}

fn display_error(err: display_interface::DisplayError) -> anyhow::Error {
    anyhow!("{:?}", err)
}


// cargo build --release --bin nothing
// cargo run --release --bin nothing
// objdump -h target/riscv32imc-esp-espidf/release/nothing
