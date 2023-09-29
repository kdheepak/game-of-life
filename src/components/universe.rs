// Based on https://rustwasm.github.io/book/game-of-life/introduction.html
use color_eyre::eyre::Result;
use itertools::Itertools;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, Frame};
use crate::{action::Action, config::Config};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
  Dead(usize),
  Alive(usize),
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
          (Cell::Alive(_), x) if x < 2 => Cell::Dead(0),
          // Rule 2: Any live cell with two or three live neighbours
          // lives on to the next generation.
          (Cell::Alive(i), 2) | (Cell::Alive(i), 3) => Cell::Alive(i.saturating_add(1)),
          // Rule 3: Any live cell with more than three live
          // neighbours dies, as if by overpopulation.
          (Cell::Alive(_), x) if x > 3 => Cell::Dead(0),
          // Rule 4: Any dead cell with exactly three live neighbours
          // becomes a live cell, as if by reproduction.
          (Cell::Dead(_), 3) => Cell::Alive(0),
          // All other cells remain in the same state.
          (Cell::Alive(i), _) => Cell::Alive(i.saturating_add(1)),
          (Cell::Dead(i), _) => Cell::Dead(i.saturating_add(1)),
        };

        next[idx] = next_cell;
      }
    }

    self.cells = next;
  }

  fn get_index(&self, row: usize, column: usize) -> usize {
    row * self.width + column
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
        count += match self.cells[idx] {
          Cell::Alive(_) => 1,
          Cell::Dead(_) => 0,
        };
      }
    }
    count
  }
}

impl Component for Universe {
  fn init(&mut self, area: Rect) -> Result<()> {
    (self.width, self.height) = (area.width as usize, area.height as usize * 2);

    self.cells = (0..self.width * self.height)
      .map(|_| if rand::random::<bool>() { Cell::Alive(0) } else { Cell::Dead(0) })
      .collect();

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
    let young = Color::Rgb(255, 213, 57);
    let old = Color::Rgb(202, 32, 77);
    let sick = Color::Reset;
    let dead = Color::Reset;
    for (y, (line1, line2)) in cells.iter().tuples().enumerate() {
      for (x, (c1, c2)) in line1.iter().zip(line2.iter()).enumerate() {
        match (c1, c2) {
          (Cell::Alive(0), Cell::Alive(0)) => {
            grid.push((x, y, '▀', Style::default().fg(young).bg(young)));
          },
          (Cell::Alive(0), Cell::Alive(_)) => {
            grid.push((x, y, '▀', Style::default().fg(young).bg(old)));
          },
          (Cell::Alive(_), Cell::Alive(0)) => {
            grid.push((x, y, '▀', Style::default().fg(old).bg(young)));
          },
          (Cell::Alive(_), Cell::Alive(_)) => {
            grid.push((x, y, '▀', Style::default().fg(old).bg(old)));
          },
          (Cell::Dead(0), Cell::Alive(0)) => {
            grid.push((x, y, '▄', Style::default().bg(sick).fg(young)));
          },
          (Cell::Dead(_), Cell::Alive(0)) => {
            grid.push((x, y, '▄', Style::default().bg(dead).fg(young)));
          },
          (Cell::Dead(0), Cell::Alive(_)) => {
            grid.push((x, y, '▄', Style::default().bg(sick).fg(old)));
          },
          (Cell::Dead(_), Cell::Alive(_)) => {
            grid.push((x, y, '▄', Style::default().bg(dead).fg(old)));
          },
          (Cell::Alive(0), Cell::Dead(0)) => {
            grid.push((x, y, '▀', Style::default().fg(young).bg(sick)));
          },
          (Cell::Alive(0), Cell::Dead(_)) => {
            grid.push((x, y, '▀', Style::default().fg(young).bg(dead)));
          },
          (Cell::Alive(_), Cell::Dead(0)) => {
            grid.push((x, y, '▀', Style::default().fg(old).bg(sick)));
          },
          (Cell::Alive(_), Cell::Dead(_)) => {
            grid.push((x, y, '▀', Style::default().fg(old).bg(dead)));
          },
          (Cell::Dead(0), Cell::Dead(0)) => {
            grid.push((x, y, ' ', Style::default().fg(sick).bg(sick)));
          },
          (Cell::Dead(0), Cell::Dead(_)) => {
            grid.push((x, y, ' ', Style::default().fg(sick).bg(dead)));
          },
          (Cell::Dead(_), Cell::Dead(0)) => {
            grid.push((x, y, ' ', Style::default().fg(dead).bg(sick)));
          },
          (Cell::Dead(_), Cell::Dead(_)) => {
            grid.push((x, y, ' ', Style::default().fg(dead).bg(dead)));
          },
        }
      }
    }
    f.render_widget(Grid { grid }, area);
    Ok(())
  }
}

struct Grid {
  grid: Vec<(usize, usize, char, Style)>,
}

impl Widget for Grid {
  fn render(self, area: Rect, buf: &mut Buffer) {
    for (x, y, ch, style) in self.grid.iter() {
      if *x >= area.width as usize || *y >= area.height as usize {
        continue;
      }
      let x = area.left() + *x as u16;
      let y = area.top() + *y as u16;
      let cell = buf.get_mut(x, y);
      cell.set_char(*ch);
      cell.set_style(*style);
    }
  }
}
