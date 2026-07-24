mod artnet;
mod config;
mod fixture;
mod ui;

use std::collections::HashMap;

use artnet::ArtDmx;
use config::Config;
use fixture::{build_rig, Fixture, State};
use iced::{
    stream, widget::{button, container, text, Column, Row},
    Color, Element, Length, Subscription, Task, Theme,
};
use tokio::net::UdpSocket;
use ui::config_panel::{ConfigEditor, FixtureField};
use ui::{config_panel, grid, stage_view};

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title(|state: &App| {
            let total = state
                .fixture_groups
                .values()
                .map(|g| g.len())
                .sum::<usize>();
            let base = format!("GOLC Mock Rig — {total} fixtures");
            match &state.last_packet {
                Some(pkt) => format!("{base} · universe {}", pkt.universe),
                None => base,
            }
        })
        .subscription(App::subscription)
        .theme(|state: &App| {
            if state.dark_mode {
                Theme::CatppuccinMocha
            } else {
                Theme::CatppuccinLatte
            }
        })
        .window_size((1024.0, 768.0))
        .run()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    Grid,
    Stage,
}

#[derive(Debug, Clone)]
struct PacketInfo {
    universe: u16,
    sequence: u8,
    physical: u8,
    length: u16,
}

struct App {
    fixture_groups: HashMap<u16, Vec<(Fixture, State)>>,
    universe_order: Vec<u16>,
    active_universe: u16,
    config: Config,
    edit_config: Option<ConfigEditor>,
    last_sender: String,
    last_packet: Option<PacketInfo>,
    dark_mode: bool,
    view_mode: ViewMode,
}

#[derive(Debug, Clone)]
enum Message {
    Dmx {
        packet: ArtDmx,
        sender: String,
    },
    ToggleTheme,
    SelectUniverse(u16),
    SwitchView(ViewMode),
    ToggleConfig,
    ConfigAddUniverse,
    ConfigRemoveUniverse(usize),
    ConfigAddFixture(usize),
    ConfigRemoveFixture(usize, usize),
    ConfigSelectUniverse(Option<usize>),
    ConfigEditFixtureField(usize, usize, FixtureField),
    ConfigNewUniverseNum(String),
    ConfigNewFixtureName(String),
    ConfigNewFixtureAddr(String),
    ConfigNewFixtureProfile(String),
    ConfigSave,
    ConfigDiscard,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let config = Config::load();
        let mut app = Self {
            fixture_groups: HashMap::new(),
            universe_order: Vec::new(),
            active_universe: 0,
            config,
            edit_config: None,
            last_sender: String::new(),
            last_packet: None,
            dark_mode: true,
            view_mode: ViewMode::Grid,
        };
        app.rebuild();
        (app, Task::none())
    }

    fn rebuild(&mut self) {
        let rig = build_rig(&self.config);
        let mut groups: HashMap<u16, Vec<(Fixture, State)>> = HashMap::new();
        for f in rig {
            groups
                .entry(f.universe)
                .or_default()
                .push((f, State::default()));
        }
        for universe in &self.config.universes {
            groups.entry(universe.num).or_default();
        }
        let mut nums: Vec<u16> = groups.keys().copied().collect();
        nums.sort();
        self.universe_order = nums;
        self.fixture_groups = groups;
        if !self.universe_order.contains(&self.active_universe) {
            self.active_universe = self.universe_order.first().copied().unwrap_or(0);
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Dmx { packet, sender } => {
                if let Some(group) = self.fixture_groups.get_mut(&packet.universe) {
                    for (fixture, state) in group.iter_mut() {
                        *state = fixture.read_state(&packet);
                    }
                }
                self.last_sender = sender;
                self.last_packet = Some(PacketInfo {
                    universe: packet.universe,
                    sequence: packet.sequence,
                    physical: packet.physical,
                    length: packet.length,
                });
                if !self.universe_order.contains(&packet.universe) {
                    self.universe_order.push(packet.universe);
                    self.universe_order.sort();
                    self.fixture_groups
                        .entry(packet.universe)
                        .or_default();
                }
            }
            Message::ToggleTheme => {
                self.dark_mode = !self.dark_mode;
            }
            Message::SelectUniverse(uni) => {
                self.active_universe = uni;
                self.fixture_groups.entry(uni).or_default();
                if !self.universe_order.contains(&uni) {
                    self.universe_order.push(uni);
                    self.universe_order.sort();
                }
            }
            Message::SwitchView(mode) => {
                self.view_mode = mode;
            }
            Message::ToggleConfig => {
                if self.edit_config.is_some() {
                    self.edit_config = None;
                } else {
                    let editor = ConfigEditor::new(self.config.clone());
                    self.edit_config = Some(editor);
                }
            }
            Message::ConfigAddUniverse => {
                if let Some(editor) = &mut self.edit_config {
                    if let Ok(num) = editor.new_universe_buf.parse::<u16>() {
                        if !editor.config.universes.iter().any(|u| u.num == num) {
                            editor
                                .config
                                .universes
                                .push(config::UniverseDef { num, fixtures: vec![] });
                            editor.selected_universe =
                                Some(editor.config.universes.len() - 1);
                        }
                        editor.new_universe_buf.clear();
                    }
                }
            }
            Message::ConfigRemoveUniverse(idx) => {
                if let Some(editor) = &mut self.edit_config {
                    if idx < editor.config.universes.len() {
                        editor.config.universes.remove(idx);
                        if editor.selected_universe == Some(idx) {
                            editor.selected_universe = None;
                        } else if let Some(sel) = editor.selected_universe {
                            if sel > idx {
                                editor.selected_universe = Some(sel - 1);
                            }
                        }
                    }
                }
            }
            Message::ConfigAddFixture(uni_idx) => {
                if let Some(editor) = &mut self.edit_config {
                    if let Some(universe) = editor.config.universes.get_mut(uni_idx) {
                        let name = std::mem::take(&mut editor.new_fixture_name_buf);
                        let addr_str = std::mem::take(&mut editor.new_fixture_addr_buf);
                        let profile = std::mem::take(&mut editor.new_fixture_profile_buf);
                        if !profile.is_empty() {
                            let address = addr_str.parse::<u16>().unwrap_or(1);
                            universe.fixtures.push(config::FixtureDef {
                                name: if name.is_empty() {
                                    format!("Fixture {}", universe.fixtures.len() + 1)
                                } else {
                                    name
                                },
                                address,
                                profile,
                                x: None,
                                y: None,
                            });
                            editor.selected_fixture =
                                Some(universe.fixtures.len() - 1);
                        }
                    }
                }
            }
            Message::ConfigRemoveFixture(uni_idx, fi_idx) => {
                if let Some(editor) = &mut self.edit_config {
                    if let Some(universe) = editor.config.universes.get_mut(uni_idx) {
                        if fi_idx < universe.fixtures.len() {
                            universe.fixtures.remove(fi_idx);
                            if editor.selected_fixture == Some(fi_idx) {
                                editor.selected_fixture = None;
                            } else if let Some(sel) = editor.selected_fixture {
                                if sel > fi_idx {
                                    editor.selected_fixture = Some(sel - 1);
                                }
                            }
                        }
                    }
                }
            }
            Message::ConfigSelectUniverse(sel) => {
                if let Some(editor) = &mut self.edit_config {
                    editor.selected_universe = sel;
                    editor.selected_fixture = None;
                }
            }
            Message::ConfigEditFixtureField(uni_idx, fi_idx, field) => {
                if let Some(editor) = &mut self.edit_config {
                    if let Some(universe) = editor.config.universes.get_mut(uni_idx) {
                        if let Some(fixture) = universe.fixtures.get_mut(fi_idx) {
                            match field {
                                FixtureField::Name(name) => fixture.name = name,
                                FixtureField::Address(addr) => {
                                    if let Ok(a) = addr.parse::<u16>() {
                                        fixture.address = a;
                                    }
                                }
                                FixtureField::Profile(prof) => fixture.profile = prof,
                            }
                        }
                    }
                }
            }
            Message::ConfigNewUniverseNum(s) => {
                if let Some(editor) = &mut self.edit_config {
                    editor.new_universe_buf = s;
                }
            }
            Message::ConfigNewFixtureName(s) => {
                if let Some(editor) = &mut self.edit_config {
                    editor.new_fixture_name_buf = s;
                }
            }
            Message::ConfigNewFixtureAddr(s) => {
                if let Some(editor) = &mut self.edit_config {
                    editor.new_fixture_addr_buf = s;
                }
            }
            Message::ConfigNewFixtureProfile(s) => {
                if let Some(editor) = &mut self.edit_config {
                    editor.new_fixture_profile_buf = s;
                }
            }
            Message::ConfigSave => {
                if let Some(editor) = self.edit_config.take() {
                    self.config = editor.config;
                    if let Err(e) = self.config.save() {
                        eprintln!("Failed to save config: {e}");
                    }
                    self.rebuild();
                }
            }
            Message::ConfigDiscard => {
                self.edit_config = None;
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        if let Some(editor) = &self.edit_config {
            return config_panel::render_config_panel(editor);
        }

        let status_line = match &self.last_packet {
            Some(pkt) => text(format!(
                "From {} — Universe {} — Seq {}, Phys {} — {} channels",
                self.last_sender,
                pkt.universe,
                pkt.sequence,
                pkt.physical,
                pkt.length
            ))
            .size(13),
            None => {
                text("Listening on UDP 0.0.0.0:6454 (Art-Net) — no packets received yet")
                    .size(13)
            }
        };

        let theme_label = if self.dark_mode { "Dark" } else { "Light" };
        let theme_btn =
            button(text(format!("{theme_label}"))).on_press(Message::ToggleTheme);
        let config_btn = button(text("Config")).on_press(Message::ToggleConfig);

        let mut tab_row = Row::new().spacing(4);
        for &uni in &self.universe_order {
            let label = format!("U{uni}");
            let is_active = uni == self.active_universe;
            let mut btn = button(text(label).size(13));
            if is_active {
                btn = btn.style(|theme: &Theme, status: button::Status| {
                    let palette = theme.palette();
                    let bg = match status {
                        button::Status::Active => palette.primary,
                        button::Status::Hovered => {
                            let p = palette.primary;
                            Color::from_rgba(p.r * 0.8, p.g * 0.8, p.b * 0.8, p.a)
                        }
                        button::Status::Pressed => {
                            let p = palette.primary;
                            Color::from_rgba(p.r * 0.6, p.g * 0.6, p.b * 0.6, p.a)
                        }
                        button::Status::Disabled => palette.background,
                    };
                    button::Style {
                        background: Some(iced::Background::Color(bg)),
                        text_color: palette.text,
                        ..Default::default()
                    }
                });
            }
            tab_row = tab_row.push(btn.on_press(Message::SelectUniverse(uni)));
        }

        let view_grid = button(text("Grid").size(13))
            .on_press(Message::SwitchView(ViewMode::Grid));
        let view_stage = button(text("Stage").size(13))
            .on_press(Message::SwitchView(ViewMode::Stage));

        let header = Column::new()
            .push(
                Row::new()
                    .push(text("GOLC Mock Rig").size(22))
                    .push(theme_btn)
                    .push(config_btn)
                    .spacing(10),
            )
            .push(
                Row::new()
                    .push(status_line)
                    .push(view_grid)
                    .push(view_stage)
                    .spacing(10),
            )
            .push(tab_row)
            .spacing(6);

        let fixtures = self
            .fixture_groups
            .get(&self.active_universe)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        let body: Element<'_, Message> = match self.view_mode {
            ViewMode::Grid => grid::render_grid(fixtures),
            ViewMode::Stage => stage_view::render_stage_view(fixtures),
        };

        let content = Column::new()
            .push(header)
            .push(body)
            .spacing(8)
            .padding(16);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::run(artnet_listener)
    }
}

fn artnet_listener() -> impl iced::futures::Stream<Item = Message> {
    stream::channel(1024, |mut output: futures::channel::mpsc::Sender<Message>| async move {
        let socket = UdpSocket::bind("0.0.0.0:6454")
            .await
            .expect("bind UDP 6454");
        let mut buf = vec![0u8; 530];
        loop {
            match socket.recv_from(&mut buf).await {
                Ok((received_len, sender_addr)) => {
                    if received_len > buf.len() {
                        continue;
                    }
                    let slice = &buf[..received_len];
                    match ArtDmx::decode(slice) {
                        Some(packet) => {
                            let sender = sender_addr.ip().to_string();
                            let result =
                                output.try_send(Message::Dmx { packet, sender });
                            if result.is_err() {
                                break;
                            }
                        }
                        None => continue,
                    }
                }
                Err(receive_error) => {
                    eprintln!("UDP receive error: {receive_error}");
                    break;
                }
            }
        }
    })
}
