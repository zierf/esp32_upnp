//! Blinks an LED
//!
//! This assumes that a LED is connected to GPIO4.
//! Depending on your target and the board you are using you should change the pin.
//! If your board doesn't have on-board LEDs don't forget to add an appropriate resistor.

mod http_client;

// If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_idf_sys as _;

use std::str::FromStr;
use std::time::Duration;
use std::{net::Ipv4Addr, thread};

use embedded_svc::wifi::{
    AuthMethod, ClientConfiguration as WifiClientConfiguration, Configuration as WifiConfiguration,
    Wifi,
};
use esp_idf_hal::{gpio::PinDriver, peripherals::Peripherals};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop, log::EspLogger, nvs::EspDefaultNvsPartition, wifi::EspWifi,
};

#[allow(unused)]
use log::{error, info, warn, LevelFilter};

#[toml_cfg::toml_config]
/// Load file cfg.toml into constant `CONFIG`.
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_password: &'static str,
    #[default("none")]
    wifi_auth_method: &'static str,
    #[default(0)]
    wifi_channel: u8,
}

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();

    // bind the log crate to ESP logging
    EspLogger::initialize_default();
    log::set_max_level(LevelFilter::max());

    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let pins = peripherals.pins;
    let mut led = PinDriver::output(pins.gpio4)?;

    let mut wifi_driver = EspWifi::new(peripherals.modem, sys_loop, Some(nvs)).unwrap();

    wifi_driver
        .set_configuration(&WifiConfiguration::Client(WifiClientConfiguration {
            ssid: CONFIG.wifi_ssid.into(),
            password: CONFIG.wifi_password.into(),
            auth_method: AuthMethod::from_str(CONFIG.wifi_auth_method)?,
            channel: Some(CONFIG.wifi_channel).and_then(|nr| if nr == 0 { None } else { Some(nr) }),
            ..Default::default()
        }))
        .unwrap();

    wifi_driver.start()?;
    wifi_driver.connect()?;

    // blink while connecting
    while !wifi_driver.is_connected()? {
        led.set_high()?;
        thread::sleep(Duration::from_millis(500));
        led.set_low()?;
        thread::sleep(Duration::from_millis(200));
    }

    let mut ip_address: Ipv4Addr = wifi_driver.sta_netif().get_ip_info()?.ip;

    // wait for ip address
    while ip_address.is_unspecified() {
        led.set_high()?;
        thread::sleep(Duration::from_millis(200));

        ip_address = wifi_driver.sta_netif().get_ip_info()?.ip;
        led.set_low()?;
    }

    // connection established, keep led on
    led.set_high()?;

    info!("Wifi ready: {:#?}", wifi_driver.sta_netif().get_ip_info()?);

    // create HTTP(S) client
    let mut client = http_client::create_client()?;

    info!("simple get request");
    let _ = http_client::get_request(&mut client);

    info!("simple post request");
    let _ = http_client::post_request(&mut client);

    loop {
        // thread::sleep to make sure the watchdog won't trigger
        thread::sleep(Duration::from_millis(5000));
    }
}
