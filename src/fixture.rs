use std::collections::HashMap;

use crate::artnet::ArtDmx;
use crate::config::Config;

#[derive(Debug, Clone)]
pub enum Channel {
    Dimmer,
    Red,
    Green,
    Blue,
    White,
    Amber,
    UV,
    Pan,
    PanFine,
    Tilt,
    TiltFine,
    ColorWheel,
    GoboWheel,
    Shutter,
    Zoom,
    Focus,
    Frost,
    Iris,
    Prism,
    EffectSpeed,
    Custom(String),
}

impl Channel {
    pub fn from_str(s: &str) -> Self {
        match s {
            "dimmer" => Self::Dimmer,
            "red" => Self::Red,
            "green" => Self::Green,
            "blue" => Self::Blue,
            "white" => Self::White,
            "amber" => Self::Amber,
            "uv" => Self::UV,
            "pan" => Self::Pan,
            "pan_fine" => Self::PanFine,
            "tilt" => Self::Tilt,
            "tilt_fine" => Self::TiltFine,
            "color_wheel" => Self::ColorWheel,
            "gobo_wheel" => Self::GoboWheel,
            "shutter" => Self::Shutter,
            "zoom" => Self::Zoom,
            "focus" => Self::Focus,
            "frost" => Self::Frost,
            "iris" => Self::Iris,
            "prism" => Self::Prism,
            "effect_speed" => Self::EffectSpeed,
            other => Self::Custom(other.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Fixture {
    pub name: String,
    pub address: u16,
    pub universe: u16,
    pub channels: Vec<Channel>,
    pub profile_name: String,
    pub pan_range: [u16; 2],
    pub tilt_range: [u16; 2],
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub color_wheel: Vec<WheelEntry>,
    pub gobo_wheel: Vec<WheelEntry>,
}

#[derive(Debug, Clone)]
pub struct WheelEntry {
    pub value: u8,
    pub color: Option<String>,
    pub label: String,
}

#[derive(Debug, Clone, Default)]
pub struct State {
    pub dimmer: f32,
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub white: f32,
    pub amber: f32,
    pub uv: f32,
    pub pan: u16,
    pub tilt: u16,
    pub color_wheel: u8,
    pub gobo_wheel: u8,
    pub shutter: f32,
    pub zoom: f32,
    pub focus: f32,
    pub frost: f32,
    pub iris: f32,
    pub prism: f32,
    pub effect_speed: f32,
    pub custom: HashMap<String, f32>,
}

impl State {
    pub fn to_rgb(&self) -> (u8, u8, u8) {
        let d = self.dimmer;
        let r = (self.red * d * 255.0).clamp(0.0, 255.0) as u8;
        let g = (self.green * d * 255.0).clamp(0.0, 255.0) as u8;
        let b = (self.blue * d * 255.0).clamp(0.0, 255.0) as u8;
        (r, g, b)
    }

    pub fn greyscale(&self) -> u8 {
        (self.dimmer * 255.0).clamp(0.0, 255.0) as u8
    }

    pub fn pan_degrees(&self, range: [u16; 2]) -> f32 {
        let span = (range[1] as f32 - range[0] as f32).max(1.0);
        range[0] as f32 + (self.pan as f32 / 65535.0) * span
    }

    pub fn tilt_degrees(&self, range: [u16; 2]) -> f32 {
        let span = (range[1] as f32 - range[0] as f32).max(1.0);
        range[0] as f32 + (self.tilt as f32 / 65535.0) * span
    }
}

impl Fixture {
    pub fn read_state(&self, dmx: &ArtDmx) -> State {
        let mut s = State::default();
        let base = (self.address.saturating_sub(1)) as usize;
        for (i, channel) in self.channels.iter().enumerate() {
            let idx = base + i;
            if idx >= 512 {
                break;
            }
            let raw = dmx.data[idx];
            match channel {
                Channel::Dimmer => s.dimmer = raw as f32 / 255.0,
                Channel::Red => s.red = raw as f32 / 255.0,
                Channel::Green => s.green = raw as f32 / 255.0,
                Channel::Blue => s.blue = raw as f32 / 255.0,
                Channel::White => s.white = raw as f32 / 255.0,
                Channel::Amber => s.amber = raw as f32 / 255.0,
                Channel::UV => s.uv = raw as f32 / 255.0,
                Channel::Pan => {
                    s.pan = (s.pan & 0x00FF) | ((raw as u16) << 8);
                }
                Channel::PanFine => {
                    s.pan = (s.pan & 0xFF00) | (raw as u16);
                }
                Channel::Tilt => {
                    s.tilt = (s.tilt & 0x00FF) | ((raw as u16) << 8);
                }
                Channel::TiltFine => {
                    s.tilt = (s.tilt & 0xFF00) | (raw as u16);
                }
                Channel::ColorWheel => s.color_wheel = raw,
                Channel::GoboWheel => s.gobo_wheel = raw,
                Channel::Shutter => s.shutter = raw as f32 / 255.0,
                Channel::Zoom => s.zoom = raw as f32 / 255.0,
                Channel::Focus => s.focus = raw as f32 / 255.0,
                Channel::Frost => s.frost = raw as f32 / 255.0,
                Channel::Iris => s.iris = raw as f32 / 255.0,
                Channel::Prism => s.prism = raw as f32 / 255.0,
                Channel::EffectSpeed => s.effect_speed = raw as f32 / 255.0,
                Channel::Custom(id) => {
                    s.custom.insert(id.clone(), raw as f32 / 255.0);
                }
            }
        }
        if s.dimmer == 0.0 {
            s.dimmer = 1.0;
        }
        s
    }
}

pub fn build_rig(config: &Config) -> Vec<Fixture> {
    let mut fixtures = Vec::new();
    for universe in &config.universes {
        for def in &universe.fixtures {
            let profile = match config.profiles.get(&def.profile) {
                Some(p) => p,
                None => continue,
            };
            let channels: Vec<Channel> =
                profile.channels.iter().map(|s| Channel::from_str(s)).collect();
            fixtures.push(Fixture {
                name: def.name.clone(),
                address: def.address,
                universe: universe.num,
                channels,
                profile_name: def.profile.clone(),
                pan_range: profile.pan_range.unwrap_or([0, 360]),
                tilt_range: profile.tilt_range.unwrap_or([0, 180]),
                x: def.x,
                y: def.y,
                color_wheel: profile
                    .color_wheel
                    .iter()
                    .map(|ws| WheelEntry {
                        value: ws.value,
                        color: ws.color.clone(),
                        label: ws.label.clone(),
                    })
                    .collect(),
                gobo_wheel: profile
                    .gobo_wheel
                    .iter()
                    .map(|ws| WheelEntry {
                        value: ws.value,
                        color: ws.color.clone(),
                        label: ws.label.clone(),
                    })
                    .collect(),
            });
        }
    }
    fixtures
}
