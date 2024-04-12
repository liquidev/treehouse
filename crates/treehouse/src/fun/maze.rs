use std::{collections::HashSet, time::Duration};

use rand::{distributions::Slice, prelude::SliceRandom, random, Rng};
use wyrand::WyRand;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Cell {
    Empty,
    Solid,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Maze {
    pub cells: Vec<Cell>,
    pub width: u32,
    pub height: u32,
}

impl Maze {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            cells: vec![Cell::Solid; width as usize * height as usize],
            width,
            height,
        }
    }

    fn is_in_bounds(&self, xy: (u32, u32)) -> bool {
        let (x, y) = xy;
        x < self.width && y < self.height
    }

    fn cell_index(&self, xy: (u32, u32)) -> Option<usize> {
        if self.is_in_bounds(xy) {
            let (x, y) = xy;
            Some(x as usize + y as usize * self.width as usize)
        } else {
            None
        }
    }

    pub fn get(&self, xy: (u32, u32)) -> Option<Cell> {
        self.cell_index(xy).map(|i| self.cells[i])
    }

    pub fn set(&mut self, xy: (u32, u32), cell: Cell) {
        if let Some(i) = self.cell_index(xy) {
            self.cells[i] = cell;
        }
    }

    pub fn maze_width(&self) -> u32 {
        self.width / 2
    }

    pub fn maze_height(&self) -> u32 {
        self.height / 2
    }

    pub fn maze_cell(x: u32) -> u32 {
        x * 2 + 1
    }

    pub fn generate(&mut self, rng: &mut WyRand) {
        // This took me way too long to figure out. I'm not very good with algorithms. :(

        let init_x = Self::maze_cell(rng.gen_range(0..self.maze_width()));
        let init_y = Self::maze_cell(rng.gen_range(0..self.maze_height()));

        let mut stack = vec![(init_x, init_y)];
        self.set((init_x, init_y), Cell::Empty);

        while !stack.is_empty() {
            let (cx, cy) = *stack.last().unwrap();

            // Look at the current cell's neighbors.
            // If there are no neighbors it can go to, bail.
            let random_neighbor = {
                let mut neighbors: [(i32, i32); 4] = [(-2, 0), (2, 0), (0, -2), (0, 2)];
                neighbors.shuffle(rng);
                let neighbor = neighbors.into_iter().find(|&(dx, dy)| {
                    self.get((cx.wrapping_add_signed(dx), cy.wrapping_add_signed(dy)))
                        .unwrap_or(Cell::Empty)
                        != Cell::Empty
                });
                if neighbor.is_none() {
                    _ = stack.pop();
                }
                neighbor
            };

            if let Some((dx, dy)) = random_neighbor {
                let (nx, ny) = (cx.saturating_add_signed(dx), cy.saturating_add_signed(dy));
                stack.push((nx, ny));

                self.set((nx, ny), Cell::Empty);
                self.set(((nx + cx) / 2, (ny + cy) / 2), Cell::Empty);
            }
        }
    }

    pub fn write_to(
        &self,
        buffer: &mut String,
        render_cell: impl Fn(u32, u32, Cell) -> &'static str,
    ) {
        for y in 0..self.height {
            for x in 0..self.width {
                buffer.push_str(render_cell(
                    x,
                    y,
                    self.cells[x as usize + y as usize * self.width as usize],
                ))
            }
            buffer.push('\n');
        }
    }

    pub fn render(&self, render_cell: impl Fn(u32, u32, Cell) -> &'static str) -> String {
        let mut s = String::new();
        self.write_to(&mut s, render_cell);
        s
    }

    pub fn render_default(&self) -> String {
        self.render(|_, _, c| match c {
            Cell::Empty => "  ",
            Cell::Solid => "##",
        })
    }
}

#[cfg(test)]
mod tests {
    use wyrand::WyRand;

    use super::Maze;

    #[test]
    fn generate() {
        let mut maze = Maze::new(33, 17);
        let mut rng = WyRand::new(1234);
        maze.generate(&mut rng);
        assert_eq!(
            maze.render_default(),
            "##################################################################
##              ##                                  ##          ##
##########  ######  ##################  ######  ##  ##########  ##
##          ##      ##              ##  ##      ##      ##      ##
##  ######  ##  ######  ##########  ##  ##  ##########  ##  ######
##      ##  ##  ##  ##      ##  ##  ##  ##  ##      ##  ##      ##
##  ##  ######  ##  ######  ##  ##########  ##  ##  ##  ######  ##
##  ##              ##      ##          ##  ##  ##              ##
##  ##################  ##########  ##  ##  ######  ##########  ##
##          ##  ##      ##          ##  ##      ##  ##      ##  ##
##########  ##  ##  ######  ##########  ##  ##  ######  ##  ######
##  ##      ##  ##      ##  ##  ##      ##  ##      ##  ##      ##
##  ##  ######  ######  ##  ##  ##  ##############  ##  ######  ##
##      ##      ##  ##  ##  ##      ##          ##      ##      ##
##  ######  ##  ##  ##  ##  ##  ######  ######  ##########  ##  ##
##          ##      ##      ##          ##                  ##  ##
##################################################################
"
        );
    }

    #[test]
    fn small() {
        let mut maze = Maze::new(9, 9);
        let mut rng = WyRand::new(217);
        maze.generate(&mut rng);
        panic!("{}", maze.render_default())
    }
}
