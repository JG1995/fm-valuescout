// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod commands;
pub mod parser;
pub use parser::parse_csv;
mod storage;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::csv_parser::parse_csv,
            commands::csv_parser::save_import,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
