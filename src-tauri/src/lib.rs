// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::fs;
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
#[tauri::command]
fn read_file() -> Result<String, String> {
    // 假設檔案放在專案根目錄 (src-tauri 同層)
    let path = std::path::Path::new("order_list.json");
    fs::read_to_string(path).map_err(|e| format!("讀檔失敗: {}", e))
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, read_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
