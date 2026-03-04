use std::fs;
use std::path::Path;

pub const APPS_DB: &str = "system/state/apps.db";
pub const PERMISSIONS_DB: &str = "system/state/permissions.db";

pub const STORE_CATALOG: [&str; 5] = [
    "org.mozilla.firefox",
    "org.libreoffice.LibreOffice",
    "com.spotify.Client",
    "org.kde.okular",
    "org.gimp.GIMP",
];

pub fn install_app(app_id: &str) -> Result<(), String> {
    if !STORE_CATALOG.contains(&app_id) {
        return Err(format!("app not in curated catalog: {app_id}"));
    }
    let mut apps = read_lines(APPS_DB)?;
    if !apps.iter().any(|entry| entry == app_id) {
        apps.push(app_id.to_string());
    }
    write_lines(APPS_DB, &apps)
}

pub fn list_installed() -> Result<Vec<String>, String> {
    read_lines(APPS_DB)
}

pub fn set_network_permission(app_id: &str, allowed: bool) -> Result<(), String> {
    let mut lines = read_lines(PERMISSIONS_DB)?;
    let key = format!("{app_id}:network=");
    lines.retain(|line| !line.starts_with(&key));
    lines.push(format!(
        "{app_id}:network={}",
        if allowed { "allow" } else { "deny" }
    ));
    write_lines(PERMISSIONS_DB, &lines)
}

pub fn network_allowed(app_id: &str) -> Result<bool, String> {
    let lines = read_lines(PERMISSIONS_DB)?;
    let key = format!("{app_id}:network=");
    for line in lines {
        if let Some(value) = line.strip_prefix(&key) {
            return Ok(value == "allow");
        }
    }
    Ok(false)
}

pub fn list_permissions() -> Result<Vec<String>, String> {
    read_lines(PERMISSIONS_DB)
}

fn read_lines(path: &str) -> Result<Vec<String>, String> {
    let p = Path::new(path);
    if !p.exists() {
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create dir failed: {e}"))?;
        }
        fs::write(path, "").map_err(|e| format!("init file failed: {e}"))?;
    }
    let content = fs::read_to_string(path).map_err(|e| format!("read file failed: {e}"))?;
    Ok(content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

fn write_lines(path: &str, lines: &[String]) -> Result<(), String> {
    let mut out = String::new();
    for line in lines {
        out.push_str(line);
        out.push('\n');
    }
    fs::write(path, out).map_err(|e| format!("write file failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_has_three_or_more_flatpaks() {
        assert!(STORE_CATALOG.len() >= 3);
    }
}
