#![no_std]
#![no_main]

use attiny_hal::Adc;
use attiny_hal::adc::AdcSettings;
use attiny_hal::clock;

use avr_device::attiny85;

use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let dp: attiny85::Peripherals = attiny85::Peripherals::take().unwrap();
    let pins: arduino_hal::Pins = arduino_hal::pins!(dp);

    let mut adc = Adc::<clock::MHz8>::new(dp.ADC, AdcSettings::default());
    let light_resistor = pins.d4.into_analog_input(&mut adc);

    let mut led = pins.d0.into_output();

    loop {
        let light: u16 = light_resistor.analog_read(&mut adc);

        let delay = 2000 - (light * 2).min(1900);

        led.set_high();
        arduino_hal::delay_ms(delay);

        led.set_low();
        arduino_hal::delay_ms(delay);
    }
}

// cargo build --release --bin blink-with-sensor
// cargo run --release --bin blink-with-sensor
// avr-size target/avr-attiny85/release/blink-with-sensor.elf
// avr-objdump -d target/avr-attiny85/release/blink-with-sensor.elf
