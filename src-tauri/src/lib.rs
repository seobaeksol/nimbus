use std::fs;

use serde::Serialize;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

enum SearchError {
    InvaildPath(String),
    IoError(String),
}

impl serde::Serialize for SearchError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let err_msg = match self {
            SearchError::InvaildPath(path) => format!("Invaild Path: {}", path),
            SearchError::IoError(reason) => format!("IO Error: {}", reason),
        };

        serializer.serialize_str(err_msg.as_str())
    }
}

impl From<std::io::Error> for SearchError {
    fn from(err: std::io::Error) -> Self {
        SearchError::IoError(err.to_string())
    }
}

#[tauri::command]
fn get_files(str_path: &str) -> Result<Vec<String>, SearchError> {
    let mut file_list = Vec::new();
    let path = std::path::PathBuf::from(str_path);

    if !path.is_dir() {
        return Err(SearchError::InvaildPath(str_path.to_string()));
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_path = entry.path();

        if let Some(file_name) = file_path.file_name() {
            if let Some(file_name_str) = file_name.to_str() {
                file_list.push(file_name_str.to_string());
            }
        }
    }

    Ok(file_list)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, get_files])
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
