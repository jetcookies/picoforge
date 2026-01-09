// use tauri::State;

mod types;
mod fido;
mod logging;
mod rescue;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logging::logger_init();
    log::info!("Initialisng PicoForge...");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            rescue::read_device_details,
            rescue::get_device_info,
            rescue::write_config,
            fido::get_fido_info,
            fido::change_fido_pin,
            fido::set_min_pin_length,
            rescue::enable_secure_boot
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
