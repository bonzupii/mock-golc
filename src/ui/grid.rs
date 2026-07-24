use crate::fixture::{Fixture, State};
use crate::ui::card;
use crate::Message;
use iced::alignment::Horizontal;
use iced::widget::{responsive, text, Column, Row};
use iced::{Element, Length};

pub fn render_grid<'a>(
    fixtures: &'a [(Fixture, State)],
) -> Element<'a, Message> {
    if fixtures.is_empty() {
        return responsive(|size| {
            let font_size = (size.width / 40.0).round().max(12.0).min(24.0);
            text("No fixtures in this universe")
                .width(Length::Fill)
                .align_x(Horizontal::Center)
                .size(font_size)
                .into()
        })
        .into();
    }

    responsive(|size| {
        let card_width = 280.0;
        let spacing = 8.0;
        let available = size.width - spacing;
        let slot = card_width + spacing;
        let per_row = (available / slot).floor() as usize;
        let per_row = per_row.max(1);

        let mut column = Column::new().spacing(spacing).padding(10).width(Length::Fill);

        for chunk in fixtures.chunks(per_row) {
            let mut row = Row::new().spacing(spacing);
            for (fixture, state) in chunk {
                row = row.push(card::render_card(fixture, state));
            }
            column = column.push(row);
        }

        column.into()
    })
    .into()
}
