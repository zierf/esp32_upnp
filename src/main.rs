//! Blinks an LED
//!
//! This assumes that a LED is connected to GPIO4.
//! Depending on your target and the board you are using you should change the pin.
//! If your board doesn't have on-board LEDs don't forget to add an appropriate resistor.

// If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_idf_sys as _;

use std::thread;
use std::time::Duration;

use esp_idf_hal::{
    gpio::{Level, PinDriver},
    peripherals::Peripherals,
};
use esp_idf_svc::log::EspLogger;

#[allow(unused)]
use log::{error, info, warn, LevelFilter};

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();

    // bind the log crate to ESP logging
    EspLogger::initialize_default();
    log::set_max_level(LevelFilter::max());

    println!("Hello, Ferris!");

    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    let mut led = PinDriver::output(pins.gpio4)?;

    loop {
        if led.is_set_low() {
            led.set_level(Level::High)?;
        } else {
            led.set_level(Level::Low)?;
        }

        // thread::sleep to make sure the watchdog won't trigger
        thread::sleep(Duration::from_millis(500));
    }
}
