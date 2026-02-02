// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command(async)]
fn long_task() -> String {
    // Simulate a long-running task
    std::thread::sleep(std::time::Duration::from_secs(5));
    "Long task completed!".to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, long_task])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
