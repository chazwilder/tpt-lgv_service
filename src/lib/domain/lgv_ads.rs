use std::env;
use std::net::Ipv4Addr;
use ads::{AdsState, AmsAddr, AmsNetId, Handle};
use ads::client::{Client as ADSClient};
use std::time::Duration;
use chrono::Local;
use crate::{ILgv};
use crate::domain::ILGVType;
use dotenvy::dotenv;
use log::info;

pub async fn get_lgv_values(i: &ILGVType) -> Result<ILgv, anyhow::Error> {
    info!("Trying PLC for {:?}", &i);
    dotenv().ok();

    let addr: Ipv4Addr = i.ams_ip.parse()?;
    let ip = addr.octets();
    let ams_net_id = AmsNetId::new(ip[0], ip[1], ip[2], ip[3], 1, 1);
    let client = ADSClient::new(
        (i.ip.clone(), ads::PORT),
        ads::Timeouts::new(Duration::from_secs(3)),
        ads::Source::Auto,
    )?;
    let device = client.device(AmsAddr::new(ams_net_id, i.ams_port));
    let state = device.get_state()?;

    if state.0 == AdsState::Run {

        let lgv_id_handle = Handle::new(device, &i.lid_tag)?;
        let lgv_id_data: i16 = lgv_id_handle.read_value()?;
        let nav_x = Handle::new(device, &i.x_tag)?;
        let nav_y = Handle::new(device, &i.y_tag)?;
        let values = Handle::new(device, &i.status_tag)?;
        let nav_x_value: f64 = nav_x.read_value()?;
        let nav_y_value: f64 = nav_y.read_value()?;
        let status_word: u16 = values.read_value()?;

    let lgv = ILgv {
            log_dttm: Local::now(),
            lgv_id: Some(lgv_id_data),
            x_pos: Some(nav_x_value),
            y_pos: Some(nav_y_value),
            reset_notify: Some(status_word & (1 << 0) != 0),
            auto_mode: Some(status_word & (1 << 1) != 0),
            loaded: Some(status_word & (1 << 2) != 0),
            in_system: Some(status_word & (1 << 3) != 0),
            position_valid: Some(status_word & (1 << 4) != 0),
            remove_block_request: Some(status_word & (1 << 5) != 0),
            local_mode: Some(status_word & (1 << 6) != 0),
            end_op_ok: Some(status_word & (1 << 7) != 0),
            moving_fw: Some(status_word & (1 << 8) != 0),
            moving_bw: Some(status_word & (1 << 9) != 0),
            waiting_for_command: Some(status_word & (1 << 10) != 0),
            on_target: Some(status_word & (1 << 11) != 0),
            end_op_fail: Some(status_word & (1 << 12) != 0),
            low_battery_alarm: Some(status_word & (1 << 13) != 0),
            agv_alarm: Some(status_word & (1 << 14) != 0),
            low_battery_warning: Some(status_word & (1 << 15) != 0),
        };
        Ok(lgv)
    } else {
        info!("PLC connection for {:?} failed.", &i);
        Ok(ILgv::default())
    }
}