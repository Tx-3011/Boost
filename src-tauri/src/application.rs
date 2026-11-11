use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Clone, Serialize)]
pub struct Application {
    pub path: String,
    pub name: String,
}

pub fn validate(path: &Path) -> Result<Application, String> {
    if !path.is_absolute() || !path.is_file() {
        return Err("Choose an existing application .exe file.".into());
    }
    let name = path.file_name().and_then(|name| name.to_str())
        .ok_or("The application filename cannot be read.")?;
    if !name.to_ascii_lowercase().ends_with(".exe") {
        return Err("Choose a Windows .exe application, not a shortcut.".into());
    }
    validate_name(name)?;
    Ok(Application { path: path.to_string_lossy().into_owned(), name: name.into() })
}

fn validate_name(name: &str) -> Result<(), String> {
    let lower = name.to_ascii_lowercase();
    if name.is_empty() || name.contains(['/', '\\', '*', '?']) {
        return Err("Choose one executable, not a wildcard or directory.".into());
    }
    if ["boost", "dispatch", "proxifyre", "proxifier"].iter().any(|part| lower.contains(part)) {
        return Err("Boost and its networking engines cannot be selected as the target app.".into());
    }
    if let Some(binary) = std::env::var_os("BOOST_DISPATCH_PATH") {
        if Path::new(&binary).file_name().is_some_and(|file| file.to_string_lossy().eq_ignore_ascii_case(name)) {
            return Err("Dispatch cannot be selected as the target app.".into());
        }
    }
    Ok(())
}

#[cfg(windows)]
pub fn choose() -> Result<Option<Application>, String> {
    use windows_sys::Win32::UI::Controls::Dialogs::{CommDlgExtendedError, GetOpenFileNameW, OPENFILENAMEW, OFN_EXPLORER, OFN_FILEMUSTEXIST, OFN_NOCHANGEDIR, OFN_PATHMUSTEXIST};
    let mut file = vec![0u16; 32768];
    let filter: Vec<u16> = "Windows applications (*.exe)\0*.exe\0\0".encode_utf16().collect();
    let title: Vec<u16> = "Choose an application to boost\0".encode_utf16().collect();
    let mut dialog: OPENFILENAMEW = unsafe { std::mem::zeroed() };
    dialog.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
    dialog.lpstrFilter = filter.as_ptr();
    dialog.lpstrFile = file.as_mut_ptr();
    dialog.nMaxFile = file.len() as u32;
    dialog.lpstrTitle = title.as_ptr();
    dialog.Flags = OFN_EXPLORER | OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR;
    // All buffers remain live for the duration of this synchronous native dialog.
    if unsafe { GetOpenFileNameW(&mut dialog) } == 0 {
        let error = unsafe { CommDlgExtendedError() };
        return if error == 0 { Ok(None) } else { Err(format!("Could not open application picker (Windows error {error}).")) };
    }
    let end = file.iter().position(|value| *value == 0).unwrap_or(file.len());
    validate(&PathBuf::from(String::from_utf16_lossy(&file[..end]))).map(Some)
}

#[cfg(not(windows))]
pub fn choose() -> Result<Option<Application>, String> { Err("Application routing currently requires Windows.".into()) }

#[cfg(test)]
mod tests {
    use super::validate_name;
    #[test]
    fn rejects_wildcards_and_network_engines() {
        for name in ["", "*", "*.exe", "dispatch.exe", "Boost.exe", "ProxiFyre.exe", "folder/app.exe"] {
            assert!(validate_name(name).is_err(), "accepted {name}");
        }
        assert!(validate_name("steam.exe").is_ok());
        assert!(validate_name("My Download App.exe").is_ok());
    }
}
