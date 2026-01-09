use ctap_hid_fido2::{Cfg, FidoKeyHidFactory};
use crate::types::FidoDeviceInfo;
use std::collections::HashMap;
// use ctap_hid_fido2::fidokey::FidoKeyHid;

#[tauri::command]
pub(crate) fn get_fido_info() -> Result<FidoDeviceInfo, String> {
    let cfg = Cfg::init();
    
    let device = FidoKeyHidFactory::create(&cfg)
        .map_err(|_| "Could not connect to FIDO device. Is it plugged in?".to_string())?;

    let info = device
        .get_info()
        .map_err(|e| format!("Error reading device info: {:?}", e))?;

    let options_map: HashMap<String, bool> = info.options.into_iter().collect();

    Ok(FidoDeviceInfo {
        versions: info.versions,
        extensions: info.extensions,
        aaguid: hex::encode_upper(info.aaguid),
        options: options_map,
        max_msg_size: info.max_msg_size,
        pin_protocols: info.pin_uv_auth_protocols,
        min_pin_length: info.min_pin_length,
        firmware_version: format!("0x{:X}", info.firmware_version),
    })
}

#[tauri::command]
pub(crate) fn change_fido_pin(
    current_pin: Option<String>,
    new_pin: String,
) -> Result<String, String> {
    let cfg = Cfg::init();
    let device = FidoKeyHidFactory::create(&cfg)
        .map_err(|e| format!("Failed to connect to FIDO device: {:?}", e))?;

    match current_pin {
        Some(old) => {
            device
                .change_pin(&old, &new_pin)
                .map_err(|e| format!("Failed to change PIN: {:?}", e))?;
            Ok("PIN Changed Successfully".into())
        }
        None => {
            device
                .set_new_pin(&new_pin)
                .map_err(|e| format!("Failed to set PIN: {:?}", e))?;
            Ok("PIN Set Successfully".into())
        }
    }
}

#[tauri::command]
pub(crate) fn set_min_pin_length(
    current_pin: String,
    min_pin_length: u8,
) -> Result<String, String> {
    let cfg = Cfg::init();
    let device = FidoKeyHidFactory::create(&cfg)
        .map_err(|e| format!("Failed to connect to FIDO device: {:?}", e))?;

    device
        .set_min_pin_length(min_pin_length, Some(&current_pin))
        .map_err(|e| format!("Failed to set minimum PIN length: {:?}", e))?;

    Ok(format!(
        "Minimum PIN length successfully set to {}",
        min_pin_length
    ))
}
