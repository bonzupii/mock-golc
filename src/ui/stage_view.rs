use crate::fixture::{Channel, Fixture, State};
use crate::Message;
use iced::mouse::Cursor;
use iced::widget::canvas::{
    Cache, Canvas, Frame, Geometry, Path, Program, Stroke,
};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Theme};

const GRID_SIZE: f32 = 20.0;

#[derive(Default)]
pub struct StageState {
    cache: Cache,
}

pub struct StageProgram<'a> {
    fixtures: &'a [(Fixture, State)],
}

impl<'a> Program<Message, Theme, Renderer> for StageProgram<'a> {
    type State = StageState;

    fn draw(
        &self,
        state: &StageState,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Vec<Geometry<Renderer>> {
        let geometry = state.cache.draw(renderer, bounds.size(), |frame: &mut Frame| {
            let w = frame.width();
            let h = frame.height();
            let bg = theme.palette().background;

            draw_grid(frame, w, h, bg);

            if self.fixtures.is_empty() {
                let cursor_pos = cursor.position_in(bounds);
                if let Some(cp) = cursor_pos {
                    let dot = Path::circle(Point::new(cp.x, cp.y), 3.0);
                    frame.fill(&dot, theme.palette().text);
                }
                return;
            }

            let has_positions = self.fixtures.iter().any(|pair| pair.0.x.is_some() && pair.0.y.is_some());

            if has_positions {
                for (fixture, state) in self.fixtures {
                    if let (Some(x), Some(y)) = (fixture.x, fixture.y) {
                        let px = (x / 100.0) * (w - 40.0) + 20.0;
                        let py = (y / 100.0) * (h - 40.0) + 20.0;
                        draw_fixture(frame, fixture, state, px, py, 8.0);
                    }
                }
            } else {
                let count = self.fixtures.len();
                if count == 0 {
                    return;
                }
                let cols = (count as f32).sqrt().ceil() as usize;
                let rows = (count + cols - 1) / cols;
                let cell_w = (w - 40.0) / cols as f32;
                let cell_h = (h - 40.0) / rows as f32;
                for (i, (fixture, state)) in self.fixtures.iter().enumerate() {
                    let (cx, cy) = auto_pos(i, cols, cell_w, cell_h);
                    let r = cell_w.min(cell_h) * 0.3;
                    draw_fixture(frame, fixture, state, cx, cy, r);
                }
                let cursor_pos = cursor.position_in(bounds);
                if let Some(cp) = cursor_pos {
                    let mut nearest_idx = 0usize;
                    let mut nearest_dist = f32::MAX;
                    for i in 0..self.fixtures.len() {
                        let (fx, fy) = auto_pos(i, cols, cell_w, cell_h);
                        let dist = ((cp.x - fx).powi(2) + (cp.y - fy).powi(2)).sqrt();
                        if dist < nearest_dist {
                            nearest_dist = dist;
                            nearest_idx = i;
                        }
                    }
                    let (nx, ny) = auto_pos(nearest_idx, cols, cell_w, cell_h);
                    let highlight = Path::circle(Point::new(nx, ny), 10.0);
                    frame.stroke(
                        &highlight,
                        Stroke::default()
                            .with_color(theme.palette().primary)
                            .with_width(2.0),
                    );
                }
            }
        });
        vec![geometry]
    }
}

fn auto_pos(i: usize, cols: usize, cell_w: f32, cell_h: f32) -> (f32, f32) {
    let col = i % cols;
    let row = i / cols;
    let cx = 20.0 + col as f32 * cell_w + cell_w / 2.0;
    let cy = 20.0 + row as f32 * cell_h + cell_h / 2.0;
    (cx, cy)
}

fn draw_grid(frame: &mut Frame, w: f32, h: f32, bg: Color) {
    let brightness = 0.299 * bg.r + 0.587 * bg.g + 0.114 * bg.b;
    let grid_color = if brightness > 0.5 {
        Color::from_rgba8(80, 80, 80, 0.2)
    } else {
        Color::from_rgba8(180, 180, 180, 0.2)
    };
    let mut x = GRID_SIZE;
    while x < w {
        let path = Path::line(Point::new(x, 0.0), Point::new(x, h));
        frame.stroke(&path, Stroke::default().with_color(grid_color));
        x += GRID_SIZE;
    }
    let mut y = GRID_SIZE;
    while y < h {
        let path = Path::line(Point::new(0.0, y), Point::new(w, y));
        frame.stroke(&path, Stroke::default().with_color(grid_color));
        y += GRID_SIZE;
    }
}

fn draw_fixture(
    frame: &mut Frame,
    fixture: &Fixture,
    state: &State,
    cx: f32,
    cy: f32,
    radius: f32,
) {
    let (ri, gi, bi) = state.to_rgb();
    let fill_color = Color::from_rgb8(ri, gi, bi);

    let is_mover = fixture
        .channels
        .iter()
        .any(|c| matches!(c, Channel::Pan));

    if is_mover {
        let pan = state.pan_degrees(fixture.pan_range);
        let radians = (pan - 90.0).to_radians();
        let beam_len = radius * 4.0;
        let end_x = cx + radians.cos() * beam_len;
        let end_y = cy + radians.sin() * beam_len;
        let beam = Path::line(Point::new(cx, cy), Point::new(end_x, end_y));
        let beam_color = Color::from_rgba8(
            (ri as f32 * 0.3) as u8,
            (gi as f32 * 0.3) as u8,
            (bi as f32 * 0.3) as u8,
            0.5,
        );
        frame.stroke(
            &beam,
            Stroke::default()
                .with_color(beam_color)
                .with_width(3.0),
        );
    }

    let circle = Path::circle(Point::new(cx, cy), radius);
    frame.fill(&circle, fill_color);

    let border = Path::circle(Point::new(cx, cy), radius);
    frame.stroke(
        &border,
        Stroke::default()
            .with_color(Color::from_rgb8(180, 180, 180))
            .with_width(1.0),
    );
}

pub fn render_stage_view<'a>(
    fixtures: &'a [(Fixture, State)],
) -> Element<'a, Message> {
    let program = StageProgram { fixtures };
    Canvas::new(program)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
