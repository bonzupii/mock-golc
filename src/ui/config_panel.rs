use crate::config::{Config, UniverseDef};
use crate::Message;
use iced::widget::{
    button, container, pick_list, row, text, text_input, Column, Row,
};
use iced::{Color, Element, Length, Theme};

pub struct ConfigEditor {
    pub config: Config,
    pub selected_universe: Option<usize>,
    pub selected_fixture: Option<usize>,
    pub new_universe_buf: String,
    pub new_fixture_name_buf: String,
    pub new_fixture_addr_buf: String,
    pub new_fixture_profile_buf: String,
    pub profile_keys: Vec<String>,
}

impl ConfigEditor {
    pub fn new(config: Config) -> Self {
        let profile_keys: Vec<String> = config.profiles.keys().cloned().collect();
        Self {
            config,
            selected_universe: None,
            selected_fixture: None,
            new_universe_buf: String::new(),
            new_fixture_name_buf: String::new(),
            new_fixture_addr_buf: String::new(),
            new_fixture_profile_buf: String::new(),
            profile_keys,
        }
    }
}

pub fn render_config_panel<'a>(
    editor: &'a ConfigEditor,
) -> Element<'a, Message> {
    let mut content = Column::new().spacing(10).padding(20);

    let title_row = row![
        text("Rig Configuration").size(20),
        button("Save").on_press(Message::ConfigSave),
        button("Discard").on_press(Message::ConfigDiscard),
    ]
    .spacing(10);

    content = content.push(title_row);

    let mut uni_col = Column::new().spacing(4);
    uni_col = uni_col.push(text("Universes").size(16));

    for (i, universe) in editor.config.universes.iter().enumerate() {
        let label = format!("Universe {}", universe.num);
        let is_selected = editor.selected_universe == Some(i);
        let mut uni_row = Row::new().spacing(8);

        let btn = if is_selected {
            button(text(label).color(Theme::CatppuccinMocha.palette().primary))
        } else {
            button(text(label))
        };
        uni_row = uni_row.push(btn.on_press(Message::ConfigSelectUniverse(Some(i))));

        uni_row = uni_row.push(
            button("X")
                .on_press(Message::ConfigRemoveUniverse(i))
                .style(|theme: &Theme, status: button::Status| {
                    let base = match status {
                        button::Status::Active => Color::from_rgb8(200, 40, 40),
                        button::Status::Hovered => Color::from_rgb8(230, 50, 50),
                        button::Status::Pressed => Color::from_rgb8(180, 30, 30),
                        button::Status::Disabled => Color::from_rgb8(100, 100, 100),
                    };
                    button::Style {
                        background: Some(iced::Background::Color(base)),
                        text_color: theme.palette().text,
                        ..Default::default()
                    }
                }),
        );

        uni_col = uni_col.push(uni_row);
    }

    let add_uni_row = row![
        text_input("Universe number...", &editor.new_universe_buf)
            .on_input(Message::ConfigNewUniverseNum)
            .width(Length::Fixed(140.0)),
        button("+").on_press(Message::ConfigAddUniverse),
    ]
    .spacing(4);
    uni_col = uni_col.push(add_uni_row);

    content = content.push(uni_col);

    if let Some(uni_idx) = editor.selected_universe {
        if let Some(universe) = editor.config.universes.get(uni_idx) {
            content = content.push(render_fixture_list(universe, uni_idx, editor));
        }
    }

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(10)
        .into()
}



fn render_fixture_list<'a>(
    universe: &'a UniverseDef,
    uni_idx: usize,
    editor: &'a ConfigEditor,
) -> Column<'a, Message> {
    let mut col = Column::new().spacing(4).padding(10);
    col = col.push(text(format!("Fixtures (Universe {})", universe.num)).size(14));

    for (fi, fixture) in universe.fixtures.iter().enumerate() {
        let is_selected = editor.selected_fixture == Some(fi);

        let mut fixture_row = Row::new().spacing(6);

        let name_input = text_input("Name", &fixture.name)
            .on_input(move |v| Message::ConfigEditFixtureField(uni_idx, fi, FixtureField::Name(v)));

        let addr_input = text_input("Addr", &fixture.address.to_string())
            .on_input(move |v| Message::ConfigEditFixtureField(uni_idx, fi, FixtureField::Address(v)));

        let profile_pick = pick_list(
            editor.profile_keys.clone(),
            Some(fixture.profile.clone()),
            move |p| Message::ConfigEditFixtureField(uni_idx, fi, FixtureField::Profile(p)),
        );

        let remove_btn = button("X")
            .on_press(Message::ConfigRemoveFixture(uni_idx, fi))
            .style(|theme: &Theme, status: button::Status| {
                let base = match status {
                    button::Status::Active => Color::from_rgb8(200, 40, 40),
                    button::Status::Hovered => Color::from_rgb8(230, 50, 50),
                    button::Status::Pressed => Color::from_rgb8(180, 30, 30),
                    button::Status::Disabled => Color::from_rgb8(100, 100, 100),
                };
                button::Style {
                    background: Some(iced::Background::Color(base)),
                    text_color: theme.palette().text,
                    ..Default::default()
                }
            });

        fixture_row = fixture_row.push(name_input).push(addr_input).push(profile_pick).push(remove_btn);

        let mut entry = Column::new().push(fixture_row);
        if is_selected {
            entry = entry.push(text("").height(Length::Fixed(4.0)));
        }

        let entry = container(entry).style(move |theme: &Theme| {
            let selection_bg = if is_selected {
                let pc = theme.palette().primary;
                Some(iced::Background::Color(Color::from_rgba(pc.r, pc.g, pc.b, 0.12)))
            } else {
                let bg = theme.palette().background;
                Some(iced::Background::Color(bg))
            };
            container::Style {
                background: selection_bg,
                ..Default::default()
            }
        });

        col = col.push(entry);
    }

    let add_row = row![
        text_input("Name", &editor.new_fixture_name_buf)
            .on_input(Message::ConfigNewFixtureName)
            .width(Length::Fixed(120.0)),
        text_input("Address", &editor.new_fixture_addr_buf)
            .on_input(Message::ConfigNewFixtureAddr)
            .width(Length::Fixed(80.0)),
        text_input("Profile", &editor.new_fixture_profile_buf)
            .on_input(Message::ConfigNewFixtureProfile)
            .width(Length::Fixed(100.0)),
        button("+").on_press(Message::ConfigAddFixture(uni_idx)),
    ]
    .spacing(4);

    col = col.push(add_row);

    col
}

#[derive(Debug, Clone)]
pub enum FixtureField {
    Name(String),
    Address(String),
    Profile(String),
}
