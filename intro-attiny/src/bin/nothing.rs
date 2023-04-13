#![no_std]
#![no_main]

// Register panic handler
use panic_halt as _;

// Do nothing. Used to set the microcontroller in a "blank state" before starting the demos.
#[arduino_hal::entry]
fn main() -> ! {
    loop {
        arduino_hal::delay_ms(1000);
    }
}

// cargo build --release --bin nothing
// cargo run --release --bin nothing
// avr-size target/avr-attiny85/release/blink.elf
// avr-objdump -d target/avr-attiny85/release/blink.elf
