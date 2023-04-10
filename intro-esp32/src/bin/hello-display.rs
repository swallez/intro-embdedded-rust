use esp_idf_sys as _;

use anyhow::anyhow;

use esp_idf_hal::i2c;
use esp_idf_hal::peripherals::Peripherals;

use anyhow::Result;

use embedded_graphics::mono_font::{ascii::FONT_10X20, MonoTextStyle};
use embedded_graphics::pixelcolor::*;
use embedded_graphics::prelude::*;
use embedded_graphics::text::*;

use ssd1306::prelude::*;

fn main() -> Result<()> {
    esp_idf_sys::link_patches(); // Will disappear once ESP-IDF 4.4

    println!("Hello, world");

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    // Initialize i2c bus for the display
    let i2c_master = i2c::Master::new(
        peripherals.i2c0,
        i2c::MasterPins { sda: pins.gpio2, scl: pins.gpio3 },
        i2c::config::MasterConfig::default(),
    )?;

    // Initialize ssd1306 display on i2c
    let display_itf = ssd1306::I2CDisplayInterface::new(i2c_master);

    let mut display = ssd1306::Ssd1306::new(
        display_itf,
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    ).into_buffered_graphics_mode();

    display.init()
        .map_err(|_| anyhow!("Display init error"))?;

    draw_text(&mut display, "Hello, world")?;

    display.flush()
        .map_err(|_| anyhow!("Display flush error"))?;

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
        Point::new(5, 40),
        MonoTextStyle::new(&FONT_10X20, BinaryColor::On),
    )
        .draw(display)
        .map_err(|_| anyhow!("Display error"))?;

    Ok(())
}


// cargo build --release --bin hello-display
// cargo run --release --bin hello-display
// objdump -h target/riscv32imc-esp-espidf/release/hello-display
