use base64::Engine;
use image::ImageFormat;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SteamGame {
    pub appid: String,
    pub name: String,
    pub last_played: u64,
    pub playtime_minutes: u64,
    pub size_on_disk: u64,
    pub icon_data_uri: String,
}

#[derive(Clone, Copy, Debug, Default)]
struct LocalAppStats {
    last_played: u64,
    playtime_minutes: u64,
}

/// Ignored non-game tools/redistributables
const IGNORED_APPIDS: &[&str] = &[
    "228980",  // Steamworks Common Redistributables
    "1070560", // Steam Linux Runtime
    "1391110", // Steam Linux Runtime - Soldier
    "1628350", // Steam Linux Runtime - Sniper
    "1493710", // Proton Experimental
];

pub fn find_steam_root() -> PathBuf {
    // 1. Check Registry HKCU\Software\Valve\Steam -> SteamPath
    if let Some(reg_path) = read_steam_path_from_registry() {
        let p = PathBuf::from(reg_path.replace('/', "\\"));
        if p.exists() {
            return p;
        }
    }

    // 2. Fallback standard locations
    let candidates = [
        r"C:\Program Files (x86)\Steam",
        r"C:\Program Files\Steam",
        r"D:\Steam",
        r"E:\Steam",
    ];
    for c in candidates {
        let p = PathBuf::from(c);
        if p.exists() {
            return p;
        }
    }

    PathBuf::from(r"C:\Program Files (x86)\Steam")
}

fn read_steam_path_from_registry() -> Option<String> {
    use windows::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, KEY_READ,
        REG_VALUE_TYPE,
    };
    use windows_core::w;

    unsafe {
        let mut hkey = HKEY::default();
        if RegOpenKeyExW(
            HKEY_CURRENT_USER,
            w!("Software\\Valve\\Steam"),
            Some(0),
            KEY_READ,
            &mut hkey,
        )
        .is_err()
        {
            return None;
        }

        let mut buf = [0u16; 512];
        let mut buf_bytes = (buf.len() * 2) as u32;
        let mut val_type = REG_VALUE_TYPE::default();
        let res = RegQueryValueExW(
            hkey,
            w!("SteamPath"),
            None,
            Some(&mut val_type),
            Some(buf.as_mut_ptr() as *mut u8),
            Some(&mut buf_bytes),
        );
        let _ = RegCloseKey(hkey);

        if res.is_ok() && buf_bytes >= 2 {
            let len = (buf_bytes as usize / 2).saturating_sub(1);
            let s = String::from_utf16_lossy(&buf[..len]);
            let trimmed = s.trim_end_matches('\0').to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
        None
    }
}

pub fn discover_library_folders(steam_root: &Path) -> Vec<PathBuf> {
    let mut folders = Vec::new();
    let mut seen = HashSet::new();

    let add_folder = |p: PathBuf, folders: &mut Vec<PathBuf>, seen: &mut HashSet<String>| {
        let key = p.to_string_lossy().to_lowercase();
        if !seen.contains(&key) && p.exists() {
            seen.insert(key);
            folders.push(p);
        }
    };

    add_folder(steam_root.to_path_buf(), &mut folders, &mut seen);

    let vdf_path = steam_root.join("steamapps").join("libraryfolders.vdf");
    if let Ok(content) = fs::read_to_string(&vdf_path) {
        for line in content.lines() {
            if let Some((k, v)) = parse_vdf_kv(line) {
                if k.eq_ignore_ascii_case("path") {
                    let unescaped = v.replace("\\\\", "\\");
                    add_folder(PathBuf::from(unescaped), &mut folders, &mut seen);
                }
            }
        }
    }

    folders
}

fn parse_vdf_kv(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if !trimmed.starts_with('"') {
        return None;
    }
    let mut parts = Vec::new();
    let mut chars = trimmed.chars().peekable();
    while let Some(&ch) = chars.peek() {
        if ch == '"' {
            chars.next();
            let mut val = String::new();
            while let Some(c) = chars.next() {
                if c == '\\' {
                    if let Some(escaped) = chars.next() {
                        val.push(escaped);
                    }
                } else if c == '"' {
                    break;
                } else {
                    val.push(c);
                }
            }
            parts.push(val);
        } else {
            chars.next();
        }
    }
    if parts.len() >= 2 {
        Some((parts[0].clone(), parts[1].clone()))
    } else {
        None
    }
}

fn parse_single_quoted_token(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with('"') || !trimmed.ends_with('"') || trimmed.len() < 3 {
        return None;
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    if inner.contains('"') {
        return None;
    }
    Some(inner.to_string())
}

/// Reads playtime (in minutes) and last played timestamps from `userdata/<id>/config/localconfig.vdf`
fn load_user_app_stats(steam_root: &Path) -> HashMap<String, LocalAppStats> {
    let mut stats_map: HashMap<String, LocalAppStats> = HashMap::new();
    let userdata_dir = steam_root.join("userdata");
    let entries = match fs::read_dir(&userdata_dir) {
        Ok(e) => e,
        Err(_) => return stats_map,
    };

    // Collect localconfig.vdf files sorted by modification time (oldest to newest so newest wins/merges)
    let mut config_files: Vec<(std::time::SystemTime, PathBuf)> = Vec::new();
    for entry in entries.flatten() {
        let cfg = entry.path().join("config").join("localconfig.vdf");
        if cfg.is_file() {
            let mtime = cfg
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            config_files.push((mtime, cfg));
        }
    }
    config_files.sort_by_key(|(t, _)| *t);

    for (_, cfg_path) in config_files {
        if let Ok(content) = fs::read_to_string(&cfg_path) {
            parse_localconfig_apps(&content, &mut stats_map);
        }
    }

    stats_map
}

fn parse_localconfig_apps(content: &str, out: &mut HashMap<String, LocalAppStats>) {
    let mut stack: Vec<String> = Vec::new();
    let mut pending_key: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "{" {
            let key = pending_key.take().unwrap_or_default();
            stack.push(key);
            continue;
        }
        if trimmed == "}" {
            pending_key = None;
            stack.pop();
            continue;
        }

        if let Some((k, v)) = parse_vdf_kv(trimmed) {
            pending_key = None;
            // Check if we are directly inside Software -> Valve -> Steam -> apps -> <appid>
            // (depth >= 2 where parent of <appid> is "apps")
            if stack.len() >= 2 && stack[stack.len() - 2].eq_ignore_ascii_case("apps") {
                let appid = &stack[stack.len() - 1];
                if appid.chars().all(|c| c.is_ascii_digit()) {
                    let entry = out.entry(appid.clone()).or_default();
                    if k.eq_ignore_ascii_case("Playtime") {
                        if let Ok(mins) = v.parse::<u64>() {
                            if mins > entry.playtime_minutes {
                                entry.playtime_minutes = mins;
                            }
                        }
                    } else if k.eq_ignore_ascii_case("LastPlayed") {
                        if let Ok(ts) = v.parse::<u64>() {
                            if ts > entry.last_played {
                                entry.last_played = ts;
                            }
                        }
                    }
                }
            }
        } else if let Some(single) = parse_single_quoted_token(trimmed) {
            pending_key = Some(single);
        }
    }
}

pub fn scan_installed_games() -> Vec<SteamGame> {
    let steam_root = find_steam_root();
    let user_stats = load_user_app_stats(&steam_root);
    let libraries = discover_library_folders(&steam_root);
    let mut games = Vec::new();
    let mut seen_appids = HashSet::new();

    for lib in libraries {
        let steamapps = lib.join("steamapps");
        let entries = match fs::read_dir(&steamapps) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n,
                None => continue,
            };

            if file_name.starts_with("appmanifest_") && file_name.ends_with(".acf") {
                if let Some(mut game) = parse_appmanifest(&path, &steam_root) {
                    if !IGNORED_APPIDS.contains(&game.appid.as_str())
                        && !seen_appids.contains(&game.appid)
                    {
                        if let Some(st) = user_stats.get(&game.appid) {
                            if st.last_played > game.last_played {
                                game.last_played = st.last_played;
                            }
                            game.playtime_minutes = st.playtime_minutes;
                        }
                        seen_appids.insert(game.appid.clone());
                        games.push(game);
                    }
                }
            }
        }
    }

    // Sort by most recently played/opened first, then by playtime, then alphabetically
    games.sort_by(|a, b| {
        b.last_played
            .cmp(&a.last_played)
            .then_with(|| b.playtime_minutes.cmp(&a.playtime_minutes))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    games
}

fn parse_appmanifest(acf_path: &Path, steam_root: &Path) -> Option<SteamGame> {
    let content = fs::read_to_string(acf_path).ok()?;
    let mut appid = String::new();
    let mut name = String::new();
    let mut last_played: u64 = 0;
    let mut size_on_disk: u64 = 0;

    for line in content.lines() {
        if let Some((k, v)) = parse_vdf_kv(line) {
            match k.as_str() {
                "appid" => appid = v,
                "name" => name = v,
                "LastPlayed" => last_played = v.parse().unwrap_or(0),
                "SizeOnDisk" => size_on_disk = v.parse().unwrap_or(0),
                _ => {}
            }
        }
    }

    if appid.is_empty() || name.is_empty() {
        return None;
    }

    let icon_data_uri = load_game_thumbnail_data_uri(steam_root, &appid, &name);

    Some(SteamGame {
        appid,
        name,
        last_played,
        playtime_minutes: 0,
        size_on_disk,
        icon_data_uri,
    })
}

fn load_game_thumbnail_data_uri(steam_root: &Path, appid: &str, game_name: &str) -> String {
    let cache_dir = steam_root
        .join("appcache")
        .join("librarycache")
        .join(appid);

    if let Some(img_path) = find_best_cached_image(&cache_dir) {
        if let Some(uri) = encode_resized_thumbnail(&img_path) {
            return uri;
        }
    }

    generate_fallback_svg_uri(game_name)
}

fn find_best_cached_image(cache_dir: &Path) -> Option<PathBuf> {
    if !cache_dir.exists() {
        return None;
    }

    let mut all_files = Vec::new();
    collect_files_recursive(cache_dir, &mut all_files);

    // Priority 1: small square icon jpg in root (typically 32x32 sha1 hash filename < 15KB)
    for f in &all_files {
        if let Some(name) = f.file_name().and_then(|n| n.to_str()) {
            let lower = name.to_lowercase();
            if lower.ends_with(".jpg")
                && !lower.starts_with("library_")
                && !lower.contains("header")
                && !lower.contains("hero")
                && !lower.contains("capsule")
            {
                return Some(f.clone());
            }
        }
    }

    // Priority 2: library_header.jpg
    for f in &all_files {
        if let Some(name) = f.file_name().and_then(|n| n.to_str()) {
            if name.eq_ignore_ascii_case("library_header.jpg") {
                return Some(f.clone());
            }
        }
    }

    // Priority 3: library_capsule.jpg or library_600x900.jpg
    for f in &all_files {
        if let Some(name) = f.file_name().and_then(|n| n.to_str()) {
            let lower = name.to_lowercase();
            if lower == "library_capsule.jpg" || lower == "library_600x900.jpg" {
                return Some(f.clone());
            }
        }
    }

    // Priority 4: logo.png
    for f in &all_files {
        if let Some(name) = f.file_name().and_then(|n| n.to_str()) {
            if name.eq_ignore_ascii_case("logo.png") {
                return Some(f.clone());
            }
        }
    }

    None
}

fn collect_files_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                collect_files_recursive(&p, out);
            } else if p.is_file() {
                out.push(p);
            }
        }
    }
}

fn encode_resized_thumbnail(img_path: &Path) -> Option<String> {
    let bytes = fs::read(img_path).ok()?;
    if bytes.len() <= 4096 {
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        return Some(format!("data:image/jpeg;base64,{}", b64));
    }

    let img = image::load_from_memory(&bytes).ok()?;
    let thumb = img.thumbnail(48, 48);
    let mut buf = Cursor::new(Vec::new());
    thumb.write_to(&mut buf, ImageFormat::Png).ok()?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(buf.into_inner());
    Some(format!("data:image/png;base64,{}", b64))
}

fn generate_fallback_svg_uri(game_name: &str) -> String {
    let initial = game_name
        .chars()
        .find(|c| c.is_alphanumeric())
        .unwrap_or('S')
        .to_uppercase()
        .to_string();
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 48 48"><rect width="48" height="48" rx="8" fill="#1b2838"/><text x="24" y="31" font-family="Segoe UI, sans-serif" font-size="22" font-weight="bold" fill="#66c0f4" text-anchor="middle">{}</text></svg>"##,
        initial
    );
    let b64 = base64::engine::general_purpose::STANDARD.encode(svg.as_bytes());
    format!("data:image/svg+xml;base64,{}", b64)
}

pub fn format_playtime(minutes: u64, strings: &crate::i18n::Strings) -> String {
    if minutes == 0 {
        let num = if strings.use_comma_decimal { "0" } else { "0" };
        format!("{} {}", num, strings.hours_unit)
    } else if minutes < 60 {
        format!("{} {}", minutes, strings.mins_unit)
    } else {
        let hours = minutes as f64 / 60.0;
        let mut formatted = format!("{:.1}", hours);
        if strings.use_comma_decimal {
            formatted = formatted.replace('.', ",");
        }
        format!("{} {}", formatted, strings.hours_unit)
    }
}

pub fn format_size(bytes: u64, strings: &crate::i18n::Strings) -> String {
    const GB: f64 = 1_073_741_824.0;
    const MB: f64 = 1_048_576.0;
    let b = bytes as f64;
    if b >= GB {
        let mut s = format!("{:.1}", b / GB);
        if strings.use_comma_decimal {
            s = s.replace('.', ",");
        }
        format!("{} GB", s)
    } else if b >= MB {
        format!("{:.0} MB", b / MB)
    } else if bytes > 0 {
        format!("{} KB", bytes / 1024)
    } else {
        strings.installed_fallback.to_string()
    }
}

pub fn launch_steam_game(appid: &str) {
    let uri = format!("steam://rungameid/{}", appid);
    open_uri(&uri);
}

pub fn open_steam_library() {
    open_uri("steam://open/games");
}

fn open_uri(uri: &str) {
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
    use windows_core::{w, HSTRING, PCWSTR};

    unsafe {
        let uri_h = HSTRING::from(uri);
        let _ = ShellExecuteW(
            None,
            w!("open"),
            PCWSTR(uri_h.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        );
    }
}
