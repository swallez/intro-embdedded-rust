#![no_std]
#![no_main]

use arduino_hal::port::{Pin, mode::Input};
use arduino_hal::port::mode::{Floating, Output};

use attiny_hal::port::PB0;

use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {

    let dp: arduino_hal::Peripherals = arduino_hal::Peripherals::take().unwrap();

    let pins = arduino_hal::Pins::with_mcu_pins(attiny_hal::Pins::new(dp.PORTB));

    let d0: Pin<Input<Floating>, PB0> = pins.d0;

    let mut led: Pin<Output, PB0> = d0.into_output();

    loop {
        led.set_high();
        arduino_hal::delay_ms(500);
        led.set_low();
        arduino_hal::delay_ms(500);
    }
}

// cargo build --release --bin blink-expanded
// cargo run --release --bin blink-expanded
// avr-size target/avr-attiny85/release/blink-expanded.elf
// avr-objdump -d target/avr-attiny85/release/blink-expanded.elf
