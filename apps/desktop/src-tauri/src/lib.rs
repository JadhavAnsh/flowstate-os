#[tauri::command]
fn protocol_version() -> u32 {
    let _ = std::mem::size_of::<flowstate_protocol::Event>();
    1
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![protocol_version])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
