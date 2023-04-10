#![no_std]
#![no_main]

// Register panic handler
use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {

    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut led = pins.d0.into_output();

    loop {
        led.set_high();
        arduino_hal::delay_ms(500);

        led.set_low();
        arduino_hal::delay_ms(500);
    }
}

// cargo build --release --bin blink
// cargo run --release --bin blink
// avr-size target/avr-attiny85/release/blink.elf
// avr-objdump -d target/avr-attiny85/release/blink.elf
