use std::{char::from_digit, fmt::Display};

use bevy::{prelude::*, reflect::Tuple};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Cell {
    #[default]
    Empty,
    Filled,
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

impl Cell {
    fn is_empty(&self) -> bool {
        matches!(self, Cell::Empty)
    }

    fn to_char(&self) -> char {
        match self {
            Self::Empty => '-',
            Self::Filled => '*',
        }
    }
}

type Clue = u8;
type Clues = Vec<Clue>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nonogram {
    #[serde(skip)]
    pub cells: Vec<Cell>,
    pub size: (u32, u32),
    pub horizontal_clues: Vec<(usize, Clues)>,
    pub vertical_clues: Vec<(usize, Clues)>,
}

impl Display for Nonogram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        let (width, height) = self.size;
        for y in 0..height {
            for x in 0..width {
                s.push(self.get((x, y)).to_char());
                s.push(' ');
            }
            if let Some((_, clue)) = self
                .horizontal_clues
                .iter()
                .find(|(pos, _)| *pos == y as usize)
            {
                clue.iter().for_each(|c| {
                    s.push(' ');
                    s.push(from_digit(*c as u32, 10).unwrap());
                });
            }
            s.push('\n');
        }
        let largest_clue = self
            .vertical_clues
            .iter()
            .map(|(_, clues)| clues)
            .max_by_key(|clues| clues.len())
            .map_or(0, |x| x.len());

        for l in 0..largest_clue {
            for x in 0..width {
                if let Some((_, clue)) = self
                    .vertical_clues
                    .iter()
                    .find(|(pos, _)| *pos == x as usize)
                {
                    s.push(from_digit(clue[l] as u32, 10).unwrap());
                    s.push(' ');
                } else {
                    s.push(' ');
                    s.push(' ');
                }
            }
            s.push('\n');
        }
        write!(f, "{}", s)
    }
}

impl Nonogram {
    // TODO get rid of unwraps
    pub fn new(
        size: (u32, u32),
        horizontal_clues: Vec<(usize, Clues)>,
        vertical_clues: Vec<(usize, Clues)>,
    ) -> Self {
        let (width, height) = size;
        Self {
            cells: vec![Cell::default(); (width * height) as usize],
            size,
            horizontal_clues,
            vertical_clues,
        }
    }

    fn pos_idx(&self, pos: (u32, u32)) -> usize {
        let width = self.size.0;
        (pos.1 * width + pos.0) as usize
    }

    pub fn get(&self, pos: (u32, u32)) -> &Cell {
        self.cells.get(self.pos_idx(pos)).unwrap()
    }

    pub fn set(&mut self, pos: (u32, u32), cell: Cell) {
        let idx = self.pos_idx(pos);
        self.cells.get_mut(idx).map(|c| core::mem::replace(c, cell));
    }

    pub fn is_valid(&self) -> bool {
        let (width, height) = self.size;
        let (width, height) = (width as usize, height as usize);
        let mut transpose = vec![Cell::default(); (width * height) as usize];
        Self::transpose(&self, &mut transpose);

        self.horizontal_clues.iter().all(|(row, clues)| {
            let offset = row * width;
            Self::verify_line(self.cells.get(offset..offset + width).unwrap(), clues)
        }) && self.vertical_clues.iter().all(|(col, clues)| {
            let offset = col * height;
            Self::verify_line(transpose.get(offset..offset + height).unwrap(), clues)
        })
    }

    fn verify_line(line: &[Cell], clues: &Clues) -> bool {
        line.split(|cell| cell.is_empty())
            .map(|s| s.len() as u8)
            .filter(|l| *l > 0)
            .collect::<Clues>()
            .eq(clues)
    }

    fn transpose(&self, output: &mut [Cell]) {
        let (width, height) = self.size;
        for x in 0..width {
            for y in 0..height {
                let idx = (x * height + y) as usize;

                *output.get_mut(idx).unwrap() = *self.cells.get(self.pos_idx((x, y))).unwrap();
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cells.iter().all(|cell| cell.is_empty())
    }
}
