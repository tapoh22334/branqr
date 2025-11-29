use std::fs;
use std::path::PathBuf;

const APP_NAME: &str = "Blanqr";
const CONFIG_FILE: &str = "config.ini";

#[derive(Clone)]
pub struct HotkeyConfig {
    pub modifiers: u32,
    pub key: u32,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        HotkeyConfig {
            modifiers: 0x0002 | 0x0004, // MOD_CONTROL | MOD_SHIFT
            key: 'B' as u32,
        }
    }
}

impl HotkeyConfig {
    pub fn display(&self) -> String {
        let mut parts = Vec::new();
        if self.modifiers & 0x0002 != 0 {
            parts.push("Ctrl");
        }
        if self.modifiers & 0x0001 != 0 {
            parts.push("Alt");
        }
        if self.modifiers & 0x0004 != 0 {
            parts.push("Shift");
        }
        if self.modifiers & 0x0008 != 0 {
            parts.push("Win");
        }
        let key_name = match self.key {
            0x41..=0x5A => {
                let c = self.key as u8 as char;
                c.to_string()
            }
            0x30..=0x39 => {
                let c = self.key as u8 as char;
                c.to_string()
            }
            0x70..=0x7B => format!("F{}", self.key - 0x6F),
            _ => format!("0x{:02X}", self.key),
        };
        parts.push(&key_name);
        parts.join("+")
    }
}

#[derive(Default)]
pub struct Config {
    pub hotkey: HotkeyConfig,
}

impl Config {
    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(content) = fs::read_to_string(&path) {
            Self::parse(&content)
        } else {
            Config::default()
        }
    }

    #[allow(dead_code)]
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = format!("hotkey = {}\n", self.hotkey.display());
        fs::write(&path, content)
    }

    pub fn config_dir() -> Option<PathBuf> {
        std::env::var("APPDATA").ok().map(|p| PathBuf::from(p).join(APP_NAME))
    }

    fn config_path() -> PathBuf {
        Self::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(CONFIG_FILE)
    }

    fn parse(content: &str) -> Self {
        let mut config = Config::default();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                if key == "hotkey" {
                    if let Some(hk) = Self::parse_hotkey(value) {
                        config.hotkey = hk;
                    }
                }
            }
        }
        config
    }

    fn parse_hotkey(s: &str) -> Option<HotkeyConfig> {
        let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
        if parts.is_empty() {
            return None;
        }

        let mut modifiers = 0u32;
        let mut key = 0u32;

        for part in &parts {
            let upper = part.to_uppercase();
            match upper.as_str() {
                "CTRL" | "CONTROL" => modifiers |= 0x0002,
                "ALT" => modifiers |= 0x0001,
                "SHIFT" => modifiers |= 0x0004,
                "WIN" | "WINDOWS" => modifiers |= 0x0008,
                _ => {
                    if upper.len() == 1 {
                        let c = upper.chars().next().unwrap();
                        if c.is_ascii_alphanumeric() {
                            key = c as u32;
                        }
                    } else if let Some(suffix) = upper.strip_prefix('F') {
                        if let Ok(n) = suffix.parse::<u32>() {
                            if (1..=12).contains(&n) {
                                key = 0x6F + n; // VK_F1 = 0x70
                            }
                        }
                    }
                }
            }
        }

        if key != 0 {
            Some(HotkeyConfig { modifiers, key })
        } else {
            None
        }
    }
}
