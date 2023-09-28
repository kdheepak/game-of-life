// Based on https://rustwasm.github.io/book/game-of-life/introduction.html
use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, Frame};
use crate::{
  action::Action,
  config::{Config, KeyBindings},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
  Dead,
  Alive,
}

#[derive(Default)]
pub struct Universe {
  command_tx: Option<UnboundedSender<Action>>,
  config: Config,
  width: usize,
  height: usize,
  cells: Vec<Cell>,
}

impl Universe {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn tick(&mut self) {
    let mut next = self.cells.clone();

    for row in 0..self.height {
      for col in 0..self.width {
        let idx = self.get_index(row, col);
        let cell = self.cells[idx];
        let live_neighbors = self.live_neighbor_count(row, col);

        let next_cell = match (cell, live_neighbors) {
          // Rule 1: Any live cell with fewer than two live neighbours
          // dies, as if caused by underpopulation.
          (Cell::Alive, x) if x < 2 => Cell::Dead,
          // Rule 2: Any live cell with two or three live neighbours
          // lives on to the next generation.
          (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
          // Rule 3: Any live cell with more than three live
          // neighbours dies, as if by overpopulation.
          (Cell::Alive, x) if x > 3 => Cell::Dead,
          // Rule 4: Any dead cell with exactly three live neighbours
          // becomes a live cell, as if by reproduction.
          (Cell::Dead, 3) => Cell::Alive,
          // All other cells remain in the same state.
          (otherwise, _) => otherwise,
        };

        next[idx] = next_cell;
      }
    }

    self.cells = next;
  }

  fn get_index(&self, row: usize, column: usize) -> usize {
    (row * self.width + column) as usize
  }

  fn live_neighbor_count(&self, row: usize, column: usize) -> u8 {
    let mut count = 0;
    for delta_row in [self.height - 1, 0, 1].iter().cloned() {
      for delta_col in [self.width - 1, 0, 1].iter().cloned() {
        if delta_row == 0 && delta_col == 0 {
          continue;
        }

        let neighbor_row = (row + delta_row) % self.height;
        let neighbor_col = (column + delta_col) % self.width;
        let idx = self.get_index(neighbor_row, neighbor_col);
        count += self.cells[idx] as u8;
      }
    }
    count
  }
}

impl Component for Universe {
  fn init(&mut self, area: Rect) -> Result<()> {
    (self.width, self.height) = (area.width as usize, area.height as usize * 2);

    self.cells =
      (0..self.width * self.height).map(|_| if rand::random::<bool>() { Cell::Alive } else { Cell::Dead }).collect();

    Ok(())
  }

  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn register_config_handler(&mut self, config: Config) -> Result<()> {
    self.config = config;
    Ok(())
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::Tick => self.tick(),
      Action::Resize(w, h) => self.init(Rect::new(0, 0, w, h))?,
      _ => {},
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
    let cells: Vec<Vec<Cell>> = self.cells.chunks(self.width).map(|chunk| chunk.to_vec()).collect();
    let mut grid = vec![];
    for (y, (line1, line2)) in cells.iter().tuples().enumerate() {
      for (x, (c1, c2)) in line1.iter().zip(line2.iter()).enumerate() {
        match (c1, c2) {
          (Cell::Alive, Cell::Alive) => {
            grid.push((x, y, '█'));
          },
          (Cell::Dead, Cell::Alive) => {
            grid.push((x, y, '▄'));
          },
          (Cell::Alive, Cell::Dead) => {
            grid.push((x, y, '▀'));
          },
          (Cell::Dead, Cell::Dead) => {
            grid.push((x, y, ' '));
          },
        }
      }
    }
    f.render_widget(Grid { grid }, area);
    Ok(())
  }
}

struct Grid {
  grid: Vec<(usize, usize, char)>,
}

impl Widget for Grid {
  fn render(self, area: Rect, buf: &mut Buffer) {
    for (x, y, ch) in self.grid.iter() {
      if *x >= area.width as usize || *y >= area.height as usize {
        continue;
      }
      let x = area.left() + *x as u16;
      let y = area.top() + *y as u16;
      let cell = buf.get_mut(x, y);
      cell.set_char(*ch);
    }
  }
}
