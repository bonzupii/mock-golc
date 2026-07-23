use crate::artnet::ArtDmx;

#[derive(Debug, Clone)]
pub enum Channel {
    Dimmer,
    Red,
    Green,
    Blue,
    White,
    Amber,
    UV,
    Custom,
}

#[derive(Debug, Clone)]
pub struct Fixture {
    pub name: String,
    pub address: u16,
    pub channels: Vec<Channel>,
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
            let raw = dmx.data[idx] as f32 / 255.0;
            match channel {
                Channel::Dimmer => s.dimmer = raw,
                Channel::Red => s.red = raw,
                Channel::Green => s.green = raw,
                Channel::Blue => s.blue = raw,
                Channel::White => s.white = raw,
                Channel::Amber => s.amber = raw,
                Channel::UV => s.uv = raw,
                Channel::Custom => {}
            }
        }
        if s.dimmer == 0.0 {
            s.dimmer = 1.0;
        }
        s
    }
}

pub fn default_rig() -> Vec<Fixture> {
    vec![
        Fixture {
            name: "Par 1".into(),
            address: 1,
            channels: vec![Channel::Dimmer, Channel::Red, Channel::Green, Channel::Blue],
        },
        Fixture {
            name: "Par 2".into(),
            address: 5,
            channels: vec![Channel::Dimmer, Channel::Red, Channel::Green, Channel::Blue],
        },
        Fixture {
            name: "Par 3".into(),
            address: 9,
            channels: vec![Channel::Dimmer, Channel::Red, Channel::Green, Channel::Blue],
        },
        Fixture {
            name: "Par 4".into(),
            address: 13,
            channels: vec![Channel::Dimmer, Channel::Red, Channel::Green, Channel::Blue],
        },
        Fixture {
            name: "Wash 1".into(),
            address: 17,
            channels: vec![
                Channel::Dimmer,
                Channel::Red,
                Channel::Green,
                Channel::Blue,
                Channel::White,
            ],
        },
        Fixture {
            name: "Wash 2".into(),
            address: 22,
            channels: vec![
                Channel::Dimmer,
                Channel::Red,
                Channel::Green,
                Channel::Blue,
                Channel::White,
            ],
        },
        Fixture {
            name: "Dimmer Rack".into(),
            address: 27,
            channels: vec![Channel::Dimmer],
        },
        Fixture {
            name: "UV Wash".into(),
            address: 28,
            channels: vec![
                Channel::Dimmer,
                Channel::Red,
                Channel::Green,
                Channel::Blue,
                Channel::UV,
            ],
        },
        Fixture {
            name: "Amber Par".into(),
            address: 33,
            channels: vec![Channel::Dimmer, Channel::Amber],
        },
        Fixture {
            name: "Strobe".into(),
            address: 35,
            channels: vec![Channel::Dimmer, Channel::Custom],
        },
    ]
}
