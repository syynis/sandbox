use std::{char::from_digit, fmt::Display};

use bevy::{prelude::*, reflect::Tuple};

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
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

#[derive(Debug)]
pub struct Nonogram<const WIDTH: usize, const HEIGHT: usize>
where
    [Cell; WIDTH * HEIGHT]: Sized,
{
    pub cells: [Cell; WIDTH * HEIGHT],
    pub horizontal_clues: Vec<(usize, Clues)>,
    pub vertical_clues: Vec<(usize, Clues)>,
}

impl<const WIDTH: usize, const HEIGHT: usize> Display for Nonogram<WIDTH, HEIGHT>
where
    [Cell; WIDTH * HEIGHT]: Sized,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                s.push(self.get((x, y)).to_char());
                s.push(' ');
            }
            if let Some((_, clue)) = self.horizontal_clues.iter().find(|(pos, _)| *pos == y) {
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
            for x in 0..WIDTH {
                if let Some((_, clue)) = self.vertical_clues.iter().find(|(pos, _)| *pos == x) {
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

impl<const WIDTH: usize, const HEIGHT: usize> Nonogram<WIDTH, HEIGHT>
where
    [Cell; WIDTH * HEIGHT]: Sized,
{
    // TODO get rid of unwraps

    pub fn new(horizontal_clues: Vec<(usize, Clues)>, vertical_clues: Vec<(usize, Clues)>) -> Self {
        Self {
            cells: [Cell::default(); WIDTH * HEIGHT],
            horizontal_clues,
            vertical_clues,
        }
    }

    fn pos_idx(pos: (usize, usize)) -> usize {
        pos.1 * WIDTH + pos.0
    }

    pub fn get(&self, pos: (usize, usize)) -> &Cell {
        self.cells.get(Self::pos_idx(pos)).unwrap()
    }

    pub fn set(&mut self, pos: (usize, usize), cell: Cell) {
        self.cells
            .get_mut(Self::pos_idx(pos))
            .map(|c| core::mem::replace(c, cell));
    }

    pub fn is_valid(&self) -> bool {
        let mut transpose = [Cell::default(); WIDTH * HEIGHT];
        Self::transpose(&self, &mut transpose);

        self.horizontal_clues.iter().all(|(row, clues)| {
            let offset = row * WIDTH;
            Self::verify_line(self.cells.get(offset..offset + WIDTH).unwrap(), clues)
        }) && self.vertical_clues.iter().all(|(col, clues)| {
            let offset = col * HEIGHT;
            Self::verify_line(transpose.get(offset..offset + HEIGHT).unwrap(), clues)
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
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                let idx = x * HEIGHT + y;

                *output.get_mut(idx).unwrap() = *self.cells.get(Self::pos_idx((x, y))).unwrap();
            }
        }
    }
}
