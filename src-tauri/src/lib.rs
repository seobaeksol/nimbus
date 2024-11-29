// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::WebviewWindowBuilder;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(generate_context())
        .expect("failed to run tauri application");
}

fn generate_context() -> tauri::Context {
    let mut context = tauri::generate_context!();
    for cmd in [
        "plugin:event|listen",
        "plugin:event|emit",
        "plugin:event|emit_to",
        "plugin:webview|create_webview_window",
    ] {
        context
            .runtime_authority_mut()
            .__allow_command(cmd.to_string(), tauri_utils::acl::ExecutionContext::Local);
    }
    context
}
