#![allow(dead_code)]

use std::{collections::HashMap, mem::swap};

use tui::style::Color;

use rand::{seq::SliceRandom, thread_rng};

type Coordinates = (usize, usize);

#[derive(Clone, Copy)]
pub struct Tetromino {
    shape: char,
    body: [Coordinates; 4],
    color: Color,
    rotation: RotationState,
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub enum ShiftError {
    BorderCollision,
    BottomCollision,
}

use crate::ui::{Playcell, Playfield};
impl Tetromino {
    fn get_length(&self) -> usize {
        let mut max = 0;
        let mut min = 0;
        for (x, _) in self.body {
            max = max.max(x);
            min = min.min(x);
        }
        max + min + 1
    }
    // Place the tetromino lowest point in the middle of the playfield's 22nd line
    pub fn spawn(&mut self, playfield: &mut Playfield) {
        let middle_point: usize = playfield.get_x_midpoint() - (self.get_length() / 2);
        // y remains unchanged, default coordinates already accommodate for it
        for (x, y) in &mut self.body {
            *x += middle_point;
            playfield.tiles[*y][*x] = Some(Playcell::new(true, self.color));
        }
    }

    pub fn place_in_playfield(self, playfield: &mut Playfield) {
        for (x, y) in self.body {
            if let Some(cell) = &mut playfield.tiles[y][x] {
                cell.is_active = false;
            }
        }
    }

    pub fn change_position(&mut self, new_body: &[Coordinates], playfield: &mut Playfield) {
        for (x, y) in &self.body {
            playfield.tiles[*y][*x] = None;
        }
        for (x, y) in new_body {
            playfield.tiles[*y][*x] = Some(Playcell::new(true, self.color));
        }
        self.body = new_body.try_into().unwrap();
    }

    pub fn collides(
        &self,
        new_body: &[Coordinates],
        playfield: &mut Playfield,
        direction: Direction,
    ) -> Result<(), ShiftError> {
        for (x, y) in new_body {
            if let Some(playcell) = playfield
                .tiles
                .get(*y)
                .ok_or(ShiftError::BottomCollision)?
                .get(*x)
                .ok_or(ShiftError::BorderCollision)?
            {
                if !playcell.is_active {
                    match direction {
                        Direction::Down => return Err(ShiftError::BottomCollision),
                        _ => return Err(ShiftError::BorderCollision),
                    }
                }
            }
        }
        Ok(())
    }

    pub fn shift(
        &mut self,
        playfield: &mut Playfield,
        direction: Direction,
    ) -> Result<(), ShiftError> {
        let mut new_body = self.body;
        match direction {
            Direction::Up => {
                for (_, y) in &mut new_body {
                    *y = y.checked_sub(1).ok_or(ShiftError::BorderCollision)?;
                }
            }
            Direction::Down => {
                for (_, y) in &mut new_body {
                    *y += 1;
                }
            }
            Direction::Right => {
                for (x, _) in &mut new_body {
                    *x = x.checked_sub(1).ok_or(ShiftError::BorderCollision)?;
                }
            }
            Direction::Left => {
                for (x, _) in &mut new_body {
                    *x += 1;
                }
            }
        };
        self.collides(&new_body, playfield, direction)?;
        self.change_position(&new_body, playfield);
        Ok(())
    }

    pub fn hard_drop(&mut self, playfield: &mut Playfield) -> ShiftError {
        let mut new_body = self.body;
        while self.collides(&new_body, playfield, Direction::Down).is_ok() {
                for (_, y) in &mut new_body {
                    *y += 1;
                }
        }
        for (_, y) in &mut new_body {
            *y -= 1;
        }
        self.change_position(&new_body, playfield);
        ShiftError::BottomCollision
    }

    pub fn rotate(&mut self, playfield: &mut Playfield, clockwise: bool) {
        if self.shape == 'O' {
            return;
        }
        // The pivot is always the first element of the body array
        let x_pivot: i32 = self.body[0].0.try_into().unwrap();
        let y_pivot: i32 = self.body[0].1.try_into().unwrap();
        let mut new_coords = Vec::new();
        for (x, y) in &mut self.body {
            let mut x_relative: i32 = *x as i32 - x_pivot;
            let mut y_relative: i32 = *y as i32 - y_pivot;
            swap(&mut x_relative, &mut y_relative);
            if clockwise {
                x_relative = -x_relative;
            } else {
                y_relative = -y_relative;
            }
            new_coords.push(((x_relative + x_pivot), (y_relative + y_pivot)));
        }
        let new_rotation = match self.rotation {
            RotationState::Normal if clockwise => RotationState::QuarterTurned,
            RotationState::Normal => RotationState::ThreeQuartersTurned,
            RotationState::QuarterTurned if clockwise => RotationState::HalfTurned,
            RotationState::QuarterTurned => RotationState::Normal,
            RotationState::HalfTurned if clockwise => RotationState::ThreeQuartersTurned,
            RotationState::HalfTurned => RotationState::QuarterTurned,
            RotationState::ThreeQuartersTurned if clockwise => RotationState::Normal,
            RotationState::ThreeQuartersTurned => RotationState::HalfTurned,
        };

        let from_rotation = if self.shape == 'I' {
            HashMap::from([
                (
                    RotationState::Normal,
                    [(0, 0), (-1, 0), (2, 0), (-1, 0), (2, 0)],
                ),
                (
                    RotationState::QuarterTurned,
                    [(-1, 0), (0, 0), (0, 0), (0, 1), (0, -2)],
                ),
                (
                    RotationState::HalfTurned,
                    [(-1, 1), (1, 1), (-2, 1), (1, 0), (-2, 0)],
                ),
                (
                    RotationState::ThreeQuartersTurned,
                    [(0, 1), (0, 1), (0, 1), (0, -1), (0, 2)],
                ),
            ])
        } else {
            HashMap::from([
                (
                    RotationState::Normal,
                    [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
                ),
                (
                    RotationState::QuarterTurned,
                    [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
                ),
                (
                    RotationState::HalfTurned,
                    [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
                ),
                (
                    RotationState::ThreeQuartersTurned,
                    [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
                ),
            ])
        };

        // Behold! A hideous piece of code
        let wallkick_tries: Vec<(i32, i32)> = from_rotation
            .get(&self.rotation)
            .unwrap()
            .iter()
            .zip(from_rotation.get(&new_rotation).unwrap())
            .map(|((x_from, y_from), (x_to, y_to))| -> (i32, i32) {
                (x_from - x_to, y_from - y_to)
            })
            .collect();

        for (x_to_try, y_to_try) in wallkick_tries {
            let to_try: Vec<(usize, usize)> = new_coords
                .iter_mut()
                .map(|(x, y)| {
                    (
                        ((*x).checked_add(x_to_try).unwrap()) as usize,
                        ((*y).checked_sub(y_to_try).unwrap()) as usize,
                    )
                })
                .collect();
            if self.collides(&to_try, playfield, Direction::Up).is_ok() {
                self.change_position(&to_try, playfield);
                self.rotation = new_rotation;
                break;
            }
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
enum RotationState {
    Normal,
    QuarterTurned,
    HalfTurned,
    ThreeQuartersTurned,
}

pub struct TetrominosBag {
    tetrominos: [Tetromino; 7],
    index: usize,
}

impl TetrominosBag {
    pub fn new() -> Self {
        Self {
            tetrominos: [
                // the first tuple is the center, necessary for rotation
                Tetromino {
                    shape: 'O',
                    body: [(0, 0), (0, 1), (1, 0), (1, 1)],
                    color: Color::Yellow,
                    rotation: RotationState::Normal,
                },
                Tetromino {
                    shape: 'I',
                    body: [(1, 0), (2, 0), (0, 0), (3, 0)],
                    color: Color::Cyan,
                    rotation: RotationState::Normal,
                },
                Tetromino {
                    shape: 'J',
                    body: [(1, 1), (0, 1), (0, 0), (2, 1)],
                    color: Color::Gray,
                    rotation: RotationState::Normal,
                },
                Tetromino {
                    shape: 'L',
                    body: [(1, 1), (0, 1), (2, 1), (2, 0)],
                    color: Color::Blue,
                    rotation: RotationState::Normal,
                },
                Tetromino {
                    shape: 'S',
                    body: [(1, 1), (0, 1), (1, 0), (2, 0)],
                    color: Color::Green,
                    rotation: RotationState::Normal,
                },
                Tetromino {
                    shape: 'Z',
                    body: [(1, 1), (1, 0), (0, 0), (2, 1)],
                    color: Color::Red,
                    rotation: RotationState::Normal,
                },
                Tetromino {
                    shape: 'T',
                    body: [(1, 1), (0, 1), (1, 0), (2, 1)],
                    color: Color::White,
                    rotation: RotationState::Normal,
                },
            ],
            index: 0,
        }
    }

    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.tetrominos.shuffle(&mut rng);
        self.index = 0;
    }

    pub fn get(&mut self) -> Tetromino {
        if self.index >= self.tetrominos.len() {
            self.shuffle();
        }
        self.index += 1;
        self.tetrominos[self.index - 1]
    }
}

impl Default for TetrominosBag {
    fn default() -> Self {
        Self::new()
    }
}

impl Playfield {
    pub fn clear_lines(&mut self) -> bool {
        let mut cleared_something = false;
        for y in 0..self.tiles.len() {
            if self.tiles[y].iter().all(|x| x.is_some()) {
                cleared_something = true;
                self.tiles[y].iter_mut().for_each(|x| *x = None);
                for line in (0..y).rev() {
                    self.tiles.swap(line, line+1);
                }
            }
        }
        cleared_something
    }
}
