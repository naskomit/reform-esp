use log::*;
use anyhow::bail;

use embedded_svc::wifi::*;
use esp_idf_svc::wifi::*;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::netif::{EspNetif, EspNetifWait};
use esp_idf_hal::peripheral;
use std::time::Duration;
use anyhow::Result;

const SSID: &str = env!("RUST_ESP32_STD_DEMO_WIFI_SSID");
const PASS: &str = env!("RUST_ESP32_STD_DEMO_WIFI_PASS");

pub fn start(
  modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
  sysloop: EspSystemEventLoop,
) -> Result<Box<EspWifi<'static>>> {
  use std::net::Ipv4Addr;

  use esp_idf_svc::handle::RawHandle;

  let mut wifi = Box::new(EspWifi::new(modem, sysloop.clone(), None)?);

  info!("Wifi created, about to scan");

  let ap_infos = wifi.scan()?;

  let ours = ap_infos.into_iter().find(|a| a.ssid == SSID);

  let channel = if let Some(ours) = ours {
      info!(
          "Found configured access point {} on channel {}",
          SSID, ours.channel
      );
      Some(ours.channel)
  } else {
      info!(
          "Configured access point {} not found during scanning, will go with unknown channel",
          SSID
      );
      None
  };

  wifi.set_configuration(&Configuration::Mixed(
      ClientConfiguration {
          ssid: SSID.into(),
          password: PASS.into(),
          channel,
          ..Default::default()
      },
      AccessPointConfiguration {
          ssid: "aptest".into(),
          channel: channel.unwrap_or(1),
          ..Default::default()
      },
  ))?;

  wifi.start()?;

  info!("Starting wifi...");

  if !WifiWait::new(&sysloop)?
      .wait_with_timeout(Duration::from_secs(20), || wifi.is_started().unwrap())
  {
      bail!("Wifi did not start");
  }

  info!("Connecting wifi...");

  wifi.connect()?;

  if !EspNetifWait::new::<EspNetif>(wifi.sta_netif(), &sysloop)?.wait_with_timeout(
      Duration::from_secs(20),
      || {
          wifi.is_connected().unwrap()
              && wifi.sta_netif().get_ip_info().unwrap().ip != Ipv4Addr::new(0, 0, 0, 0)
      },
  ) {
      bail!("Wifi did not connect or did not receive a DHCP lease");
  }

  let ip_info = wifi.sta_netif().get_ip_info()?;

  info!("Wifi DHCP info: {:?}", ip_info);

  // ping(ip_info.subnet.gateway)?;

  Ok(wifi)
}