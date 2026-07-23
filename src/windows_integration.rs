use std::{
    io,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathFormat {
    Windows,
    PowerShell,
    FileUri,
    Wsl,
}

pub fn format_path(path: &Path, format: PathFormat) -> String {
    let windows = path.to_string_lossy();
    match format {
        PathFormat::Windows => windows.into_owned(),
        PathFormat::PowerShell => format!("'{}'", windows.replace('\'', "''")),
        PathFormat::FileUri => {
            let normalized = windows.replace('\\', "/");
            if normalized.starts_with("//") {
                format!("file:{normalized}")
            } else {
                format!("file:///{}", percent_encode_path(&normalized))
            }
        }
        PathFormat::Wsl => windows_to_wsl(&windows),
    }
}

pub fn open_terminal(path: &Path) -> io::Result<()> {
    let directory = if path.is_dir() {
        path
    } else {
        path.parent().unwrap_or(path)
    };
    let mut command = Command::new("powershell.exe");
    command
        .arg("-NoExit")
        .arg("-Command")
        .arg("Set-Location -LiteralPath $args[0]")
        .arg(directory);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        command.creation_flags(CREATE_NEW_PROCESS_GROUP);
    }
    command.spawn().map(|_| ())
}

pub fn reveal_in_file_explorer(path: &Path) -> io::Result<()> {
    let mut command = Command::new("explorer.exe");
    if path.is_file() {
        command.arg(format!("/select,{}", path.display()));
    } else {
        command.arg(path);
    }
    command.spawn().map(|_| ())
}

pub fn known_folders() -> Vec<(&'static str, PathBuf)> {
    let home = std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));
    let mut folders = vec![("Home", home.clone())];
    for (label, name) in [
        ("Desktop", "Desktop"),
        ("Downloads", "Downloads"),
        ("Documents", "Documents"),
        ("Pictures", "Pictures"),
        ("Videos", "Videos"),
        ("Music", "Music"),
    ] {
        let path = home.join(name);
        if path.is_dir() {
            folders.push((label, path));
        }
    }
    folders
}

fn windows_to_wsl(path: &str) -> String {
    let bytes = path.as_bytes();
    if bytes.len() >= 3 && bytes[1] == b':' && (bytes[2] == b'\\' || bytes[2] == b'/') {
        let drive = (bytes[0] as char).to_ascii_lowercase();
        let rest = path[3..].replace('\\', "/");
        format!("/mnt/{drive}/{rest}")
    } else if path.starts_with("\\\\") {
        format!(
            "/mnt/unc/{}",
            path.trim_start_matches('\\').replace('\\', "/")
        )
    } else {
        path.replace('\\', "/")
    }
}

fn percent_encode_path(path: &str) -> String {
    let mut encoded = String::with_capacity(path.len());
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b':' | b'-' | b'_' | b'.' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_windows_paths_for_powershell_and_wsl() {
        let path = Path::new(r"C:\Work\Nimbus");
        assert_eq!(
            format_path(path, PathFormat::PowerShell),
            r"'C:\Work\Nimbus'"
        );
        assert_eq!(format_path(path, PathFormat::Wsl), "/mnt/c/Work/Nimbus");
    }
}
