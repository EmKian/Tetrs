// Here will be contained all the graphics-related stuff.

use tui::{
    backend::Backend,
    buffer::Buffer,
    layout::{Rect, Layout, Direction, Constraint},
    style::Color,
    widgets::{Block, Borders},
    Terminal,
};

pub struct Playfield {
    pub rect: Rect,
    pub tiles: Vec<Vec<Option<Playcell>>>,
    x_scaling: u16,
    y_scaling: u16,
}

#[derive(Clone, Copy)]
pub struct Playcell {
    pub is_active: bool,
    color: Color,
}

impl Playcell {
    pub fn new(is_active: bool, color: Color) -> Self {
        Self { is_active, color }
    }
}

impl Playfield {
    pub fn new(
        frame_width: u16,
        frame_height: u16,
        width: u16,
        height: u16,
        x_scaling: u16,
        y_scaling: u16,
    ) -> Self {
        const BORDER_PIXELS: u16 = 2;
        // They are centered
        let orig_x = (frame_width - width * x_scaling + 2) / 2;
        let orig_y = (frame_height - height * y_scaling + 2) / 2;
        let rect = Rect::new(
            orig_x,
            orig_y,
            width * x_scaling + BORDER_PIXELS,
            height * y_scaling + BORDER_PIXELS,
        );
        let mut tiles: Vec<Vec<Option<Playcell>>> = Vec::with_capacity(height.into());
        for _ in 0..height {
            tiles.push(vec![None; width.into()]);
        }
        Self {
            rect,
            tiles,
            x_scaling,
            y_scaling,
        }
    }

    pub fn get_x_midpoint(&self) -> usize {
        ((self.rect.width - 2) / self.x_scaling / 2).into()
    }
    pub fn draw<B: Backend>(&self, terminal: &mut Terminal<B>) {
        const _BLOCK: char = '\u{2588}';
        let playcells = &self.tiles;

        let mut buffer = Buffer::empty(self.rect);
        for y in 0..playcells.len() * usize::from(self.y_scaling) {
            for x in 0..playcells[0].len() * usize::from(self.x_scaling) {
                let cell = buffer.get_mut(x as u16 + 1 + self.rect.x, y as u16 + 1 + self.rect.y);
                if let Some(color) =
                    &playcells[y / usize::from(self.y_scaling)][x / usize::from(self.x_scaling)]
                {
                    cell.set_bg(color.color);
                }
            }
        }

        terminal.current_buffer_mut().merge(&buffer);
        terminal
            .draw(|f| {
                // let chunks = Layout::default()
                //     .direction(Direction::Horizontal)
                //     .constraints([
                //                  Constraint::Length(5),
                //                  Constraint::Min(0),
                //     ]
                //     .as_ref(),
                //     ).split(Rect { x: self.rect.x, y: self.rect.y, width: self.rect.width *2 , height: self.rect.height});
                //
                let block = Block::default().borders(Borders::ALL);
                f.render_widget(block, self.rect);
                let block2 = Block::default().borders(Borders::ALL);
                f.render_widget(block2, Rect { x: self.rect.x + self.rect.width + self.x_scaling, y: self.rect.y, width: 5 * self.x_scaling, height: 5 * self.y_scaling })
            })
            .unwrap();
    }
}
