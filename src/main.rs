mod artnet;
mod fixture;

use artnet::ArtDmx;
use fixture::{default_rig, Fixture, State};
use iced::{
    stream, widget::{button, container, row, text, Column, Row},
    Color, Element, Length, Subscription, Task, Theme,
};
use tokio::net::UdpSocket;

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .title(|state: &App| {
            let count = state.fixtures.len();
            let base = format!("GOLC Mock Rig — {count} fixtures");
            if state.last_sender.len() > 0 {
                format!("{base} · universe {}", state.last_universe)
            } else {
                base
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

struct App {
    fixtures: Vec<(Fixture, State)>,
    last_sender: String,
    last_universe: u16,
    last_sequence: u8,
    last_physical: u8,
    last_length: u16,
    dark_mode: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Dmx { packet: ArtDmx, sender: String },
    ToggleTheme,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let rig = default_rig();
        let fixtures = rig.into_iter().map(|f| (f, State::default())).collect();
        (
            App {
                fixtures,
                last_sender: String::new(),
                last_universe: 0,
                last_sequence: 0,
                last_physical: 0,
                last_length: 0,
                dark_mode: true,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Dmx { packet, sender } => {
                for (fixture, state) in &mut self.fixtures {
                    *state = fixture.read_state(&packet);
                }
                self.last_sender = sender;
                self.last_universe = packet.universe;
                self.last_sequence = packet.sequence;
                self.last_physical = packet.physical;
                self.last_length = packet.length;
            }
            Message::ToggleTheme => {
                self.dark_mode = !self.dark_mode;
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let fixture_widgets: Vec<Element<Message>> = self
            .fixtures
            .iter()
            .map(|(fixture, state)| render_fixture(fixture, state))
            .collect();

        let status_line = if self.last_sender.len() > 0 {
            text(format!(
                "From {} — Universe {} — Seq {}, Phys {} — {} channels",
                self.last_sender,
                self.last_universe,
                self.last_sequence,
                self.last_physical,
                self.last_length
            ))
            .size(14)
        } else {
            text("Listening on UDP 0.0.0.0:6454 (Art-Net) — no packets received yet").size(14)
        };

        let theme_label = if self.dark_mode { "Dark" } else { "Light" };
        let theme_button = button(text(format!("Toggle ({theme_label})"))).on_press(Message::ToggleTheme);

        let header = Column::new()
            .push(text("GOLC Mock Rig").size(24))
            .push(Row::new().push(status_line).push(theme_button).spacing(20));

        let grid = Column::with_children(fixture_widgets).spacing(8).padding(10);

        let content = Column::new()
            .push(header)
            .push(grid)
            .spacing(10)
            .padding(20);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
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

fn render_fixture<'a>(fixture: &'a Fixture, state: &'a State) -> Element<'a, Message> {
    let (red_value, green_value, blue_value) = state.to_rgb();
    let has_any_color = red_value > 0 || green_value > 0 || blue_value > 0;

    let swatch = container(
        text("")
            .width(Length::Fixed(60.0))
            .height(Length::Fixed(60.0)),
    )
    .style(move |theme: &Theme| container::Style {
        background: Some(Color::from_rgb8(red_value, green_value, blue_value).into()),
        text_color: None,
        border: iced::Border {
            color: theme.palette().text,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    });

    let dimmer_value = state.greyscale();

    let info = Column::new()
        .push(text(&fixture.name).size(16))
        .push(text(format!("Addr: {}", fixture.address)).size(12))
        .push(
            text(if has_any_color {
                format!("RGB ({red_value}, {green_value}, {blue_value})")
            } else {
                format!("Dim {dimmer_value:>3}")
            })
            .size(12),
        )
        .spacing(2);

    row![swatch, info].spacing(12).into()
}
