use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use esp_idf_hal::{prelude::Peripherals};
use anyhow::Result;
use esp_idf_svc::eventloop::EspSystemEventLoop;
mod services;

fn main() -> Result<()> {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals: Peripherals = Peripherals::take().unwrap();
    
    let sysloop = EspSystemEventLoop::take()?;

    let mut wifi = services::wifi::start(peripherals.modem, sysloop.clone())?;

    let mqtt = services::mqtt::start();

    println!("Hello, world!");
    Ok(())
}
