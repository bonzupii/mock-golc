use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub profiles: HashMap<String, ProfileDef>,
    #[serde(default)]
    pub universes: Vec<UniverseDef>,
}

impl Config {
    pub fn path() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("mock-golc").join("rig.toml")
    }

    pub fn load() -> Self {
        let path = Self::path();
        fs::read_to_string(&path)
            .ok()
            .and_then(|content| toml::from_str(&content).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, content).map_err(|e| e.to_string())
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut profiles = HashMap::new();
        profiles.insert(
            "par".into(),
            ProfileDef {
                label: "RGB Par".into(),
                channels: vec!["dimmer".into(), "red".into(), "green".into(), "blue".into()],
                pan_range: None,
                tilt_range: None,
                color_wheel: vec![],
                gobo_wheel: vec![],
            },
        );
        profiles.insert(
            "wash".into(),
            ProfileDef {
                label: "RGBW Wash".into(),
                channels: vec![
                    "dimmer".into(),
                    "red".into(),
                    "green".into(),
                    "blue".into(),
                    "white".into(),
                ],
                pan_range: None,
                tilt_range: None,
                color_wheel: vec![],
                gobo_wheel: vec![],
            },
        );
        profiles.insert(
            "mover".into(),
            ProfileDef {
                label: "Moving Head".into(),
                channels: vec![
                    "dimmer".into(),
                    "pan".into(),
                    "pan_fine".into(),
                    "tilt".into(),
                    "tilt_fine".into(),
                    "color_wheel".into(),
                    "gobo_wheel".into(),
                    "shutter".into(),
                    "zoom".into(),
                ],
                pan_range: Some([0, 540]),
                tilt_range: Some([0, 270]),
                color_wheel: vec![
                    WheelSlot {
                        value: 0,
                        color: Some("#ffffff".into()),
                        label: "Open".into(),
                    },
                    WheelSlot {
                        value: 32,
                        color: Some("#ff0000".into()),
                        label: "Red".into(),
                    },
                    WheelSlot {
                        value: 64,
                        color: Some("#00ff00".into()),
                        label: "Green".into(),
                    },
                    WheelSlot {
                        value: 96,
                        color: Some("#0000ff".into()),
                        label: "Blue".into(),
                    },
                    WheelSlot {
                        value: 128,
                        color: Some("#ffff00".into()),
                        label: "Yellow".into(),
                    },
                    WheelSlot {
                        value: 160,
                        color: Some("#ff00ff".into()),
                        label: "Magenta".into(),
                    },
                    WheelSlot {
                        value: 192,
                        color: Some("#00ffff".into()),
                        label: "Cyan".into(),
                    },
                    WheelSlot {
                        value: 224,
                        color: Some("#ffffff".into()),
                        label: "White".into(),
                    },
                ],
                gobo_wheel: vec![
                    WheelSlot {
                        value: 0,
                        color: None,
                        label: "Open".into(),
                    },
                    WheelSlot {
                        value: 32,
                        color: None,
                        label: "Gobo 1".into(),
                    },
                    WheelSlot {
                        value: 64,
                        color: None,
                        label: "Gobo 2".into(),
                    },
                    WheelSlot {
                        value: 96,
                        color: None,
                        label: "Gobo 3".into(),
                    },
                    WheelSlot {
                        value: 128,
                        color: None,
                        label: "Gobo 4".into(),
                    },
                ],
            },
        );

        let universes = vec![UniverseDef {
            num: 0,
            fixtures: vec![
                FixtureDef {
                    name: "Par 1".into(),
                    address: 1,
                    profile: "par".into(),
                    x: None,
                    y: None,
                },
                FixtureDef {
                    name: "Par 2".into(),
                    address: 5,
                    profile: "par".into(),
                    x: None,
                    y: None,
                },
                FixtureDef {
                    name: "Par 3".into(),
                    address: 9,
                    profile: "par".into(),
                    x: None,
                    y: None,
                },
                FixtureDef {
                    name: "Par 4".into(),
                    address: 13,
                    profile: "par".into(),
                    x: None,
                    y: None,
                },
                FixtureDef {
                    name: "Wash 1".into(),
                    address: 17,
                    profile: "wash".into(),
                    x: None,
                    y: None,
                },
                FixtureDef {
                    name: "Wash 2".into(),
                    address: 22,
                    profile: "wash".into(),
                    x: None,
                    y: None,
                },
                FixtureDef {
                    name: "Mover 1".into(),
                    address: 27,
                    profile: "mover".into(),
                    x: None,
                    y: None,
                },
                FixtureDef {
                    name: "Mover 2".into(),
                    address: 36,
                    profile: "mover".into(),
                    x: None,
                    y: None,
                },
            ],
        }];

        Self { profiles, universes }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniverseDef {
    pub num: u16,
    #[serde(default)]
    pub fixtures: Vec<FixtureDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureDef {
    pub name: String,
    pub address: u16,
    pub profile: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileDef {
    pub label: String,
    pub channels: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pan_range: Option<[u16; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tilt_range: Option<[u16; 2]>,
    #[serde(default)]
    pub color_wheel: Vec<WheelSlot>,
    #[serde(default)]
    pub gobo_wheel: Vec<WheelSlot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WheelSlot {
    pub value: u8,
    pub color: Option<String>,
    pub label: String,
}
