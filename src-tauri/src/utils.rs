use anyhow::Result;

pub fn normalize_url(input: &str) -> Result<tauri::Url, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("URL 不能为空".to_string());
    }
    let url = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("https://{}", trimmed)
    };
    tauri::Url::parse(&url).map_err(|e| e.to_string())
}

pub fn browser_label(session_id: &str) -> String {
    let mut label = String::from("browser-");
    for ch in session_id.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '/' | ':' | '_') {
            label.push(ch);
        } else {
            label.push('_');
        }
    }
    label
}

pub fn chrome_profile_dir(
    session_id: &str,
    profile_mode: &str,
) -> Result<std::path::PathBuf, String> {
    let base = std::env::temp_dir()
        .join("synapsh-chrome")
        .join(browser_label(session_id));
    let dir = if profile_mode == "new" {
        base.join(format!("profile-{}", uuid::Uuid::new_v4()))
    } else {
        base.join("profile")
    };
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

pub fn base64_encode(data: &[u8]) -> String {
    use base64::{engine::general_purpose, Engine as _};
    general_purpose::STANDARD.encode(data)
}

pub fn base64_decode(data: &str) -> Result<Vec<u8>, String> {
    use base64::{engine::general_purpose, Engine as _};
    general_purpose::STANDARD
        .decode(data)
        .map_err(|e| e.to_string())
}
