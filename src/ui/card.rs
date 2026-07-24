use crate::fixture::{Channel, Fixture, State, WheelEntry};
use crate::Message;
use iced::{
    widget::{container, row, text, Column},
    Color, Element, Length, Theme,
};

fn wheel_info(wheel: &[WheelEntry], value: u8) -> (String, Option<String>) {
    for entry in wheel {
        if entry.value == value {
            return (entry.label.clone(), entry.color.clone());
        }
    }
    if wheel.is_empty() {
        return (String::new(), None);
    }
    let mut closest = &wheel[0];
    for entry in wheel {
        if (entry.value as i16 - value as i16).abs()
            < (closest.value as i16 - value as i16).abs()
        {
            closest = entry;
        }
    }
    (closest.label.clone(), closest.color.clone())
}

pub fn render_card<'a>(fixture: &'a Fixture, state: &'a State) -> Element<'a, Message> {
    let (red, green, blue) = state.to_rgb();
    let has_color = red > 0 || green > 0 || blue > 0;

    let swatch = container(
        text("").width(Length::Fixed(60.0)).height(Length::Fixed(60.0)),
    )
    .style(move |theme: &Theme| container::Style {
        background: Some(Color::from_rgb8(red, green, blue).into()),
        border: iced::Border {
            color: theme.palette().text,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    });

    let dimmer_val = state.greyscale();

    let mut info = Column::new()
        .push(text(&fixture.name).size(14))
        .push(text(format!("U{} @ {} · {}", fixture.universe, fixture.address, fixture.profile_name)).size(11))
        .spacing(1);

    if has_color {
        info = info.push(
            text(format!("{}  RGB({red},{green},{blue})", dimmer_val))
                .size(11)
                .color(Color::from_rgb8(
                    (red.max(40) as f32 * 0.6) as u8,
                    (green.max(40) as f32 * 0.6) as u8,
                    (blue.max(40) as f32 * 0.6) as u8,
                )),
        );
    } else {
        info = info.push(text(format!("Dim {dimmer_val:>3}")).size(11));
    }

    let is_mover = fixture
        .channels
        .iter()
        .any(|c| matches!(c, Channel::Pan));

    if is_mover {
        let pan = state.pan_degrees(fixture.pan_range);
        let tilt = state.tilt_degrees(fixture.tilt_range);
        info = info.push(text(format!("Pan {pan:.0}°  Tilt {tilt:.0}°")).size(11));

        let (cw_label, cw_color) = wheel_info(&fixture.color_wheel, state.color_wheel);
        if !cw_label.is_empty() {
            let mut cw_text = format!("Color: {cw_label}");
            if let Some(ref c) = cw_color {
                if !c.is_empty() {
                    cw_text = format!("Color: {cw_label} [{c}]");
                }
            }
            info = info.push(text(cw_text).size(11));
        }

        let (gw_label, gw_color) = wheel_info(&fixture.gobo_wheel, state.gobo_wheel);
        if !gw_label.is_empty() {
            let mut gw_text = format!("Gobo: {gw_label}");
            if let Some(ref c) = gw_color {
                if !c.is_empty() {
                    gw_text = format!("Gobo: {gw_label} [{c}]");
                }
            }
            info = info.push(text(gw_text).size(11));
        }
    }

    row![swatch, info].spacing(10).into()
}
