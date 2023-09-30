// Based on https://github.com/splintersuidman/game-of-life/tree/master/src/lib/parsers
use std::{fs::File, io::Read};

use color_eyre::eyre::Result;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Cell {
  Dead(usize),
  Alive(usize),
}

impl From<bool> for Cell {
  fn from(value: bool) -> Self {
    if value {
      Cell::Alive(0)
    } else {
      Cell::Dead(0)
    }
  }
}

impl std::ops::Not for Cell {
  type Output = Self;

  fn not(self) -> Self::Output {
    match self {
      Cell::Dead(_) => Cell::Alive(0),
      Cell::Alive(_) => Cell::Dead(0),
    }
  }
}

impl std::fmt::Display for Cell {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match *self {
      Cell::Dead(_) => write!(f, " "),
      Cell::Alive(_) => write!(f, "●"),
    }
  }
}

/// Describes what type of file it is based on the file extension.
pub enum FileType {
  Life,
  PlainText,
  RLE,
}

impl FileType {
  /// Parses the file type from filename.
  pub fn from_filename(s: &str) -> Option<FileType> {
    if s.ends_with("lif") || s.ends_with("life") {
      Some(FileType::Life)
    } else if s.ends_with("cells") {
      Some(FileType::PlainText)
    } else if s.ends_with("rle") {
      Some(FileType::RLE)
    } else {
      None
    }
  }
}

#[derive(Default)]
pub struct Pattern {
  pub cells: Vec<(isize, isize)>,
  pub name: Option<String>,
  pub description: Option<String>,
  pub author: Option<String>,
  pub area: Option<(usize, usize)>,
}

impl Pattern {
  pub fn from_file(filename: &str) -> Result<Pattern> {
    // Read file and get rules from them.
    let mut file = match File::open(filename) {
      Ok(f) => f,
      Err(e) => return Err(color_eyre::eyre::eyre!("Could not open file: {}", e)),
    };

    let mut contents = String::new();
    if let Err(e) = file.read_to_string(&mut contents) {
      return Err(color_eyre::eyre::eyre!("Could not read file to string: {}", e));
    }

    let file_type: FileType = FileType::from_filename(filename).expect("Unrecognised file type.");

    let pattern = match file_type {
      FileType::Life => todo!("Not implemented"),
      FileType::PlainText => todo!("Not implemented"),
      FileType::RLE => parse_rle_file(&contents)?,
    };
    Ok(pattern)
  }
}

pub fn parse_rle_file(s: &str) -> Result<Pattern> {
  let s = s.to_string();
  let mut pattern: Pattern = Default::default();

  // Metadata
  let metadata = s.lines().take_while(|x| x.starts_with('#'));

  for line in metadata {
    let mut linedata = line.chars().skip(1);
    match linedata.next() {
      Some('N') => {
        // Name
        let name: String = linedata.collect();
        let name = name.trim();
        if !name.is_empty() {
          pattern.name = Some(String::from(name));
        }
      },
      Some('C') | Some('c') => {
        // Comment or description
        let description: String = linedata.collect();
        let description = description.trim();
        if let Some(d) = pattern.description {
          pattern.description = Some(format!("{}\n{}", d, description));
        } else {
          pattern.description = Some(String::from(description));
        }
      },
      Some('O') => {
        // Author
        let author: String = linedata.collect();
        let author = author.trim();
        pattern.author = Some(String::from(author));
      },
      Some(unknown_char) => {
        return Err(color_eyre::eyre::eyre!("Unknown combination #{} in metadata of .rle file.", unknown_char));
      },
      None => {},
    }
  }

  // Remove all of the lines starting with `#`
  let mut lines = s.lines().skip_while(|x| x.starts_with('#'));

  // x = m, y = n
  match lines.next() {
    Some(v) => {
      if v.contains("x = ") && v.contains("y = ") {
        let v: Vec<&str> = v.splitn(3, ", ").collect();
        let x = v[0];
        let y = v[1];
        log::info!("{x} {y}");
        let x = x.replace("x = ", "").parse::<usize>()?;
        let y = y.replace("y = ", "").parse::<usize>()?;
        pattern.area = Some((x, y));
      }
    },
    None => {
      return Err(color_eyre::eyre::eyre!(
        "The header for this `.rle` file could not be found because there were no (uncommented) lines.",
      ))
    },
  };

  // TODO: process header information

  let data: String = lines.collect();
  let data = data.split('$');

  let mut y: isize = 0;
  for line in data {
    let mut amount: isize = 0;
    let mut x = 0;
    for c in line.chars() {
      match c {
        'b' | '.' => {
          // Off state
          if amount == 0 {
            // Not preceded by a number
            x += 1;
          } else {
            x += amount;
            amount = 0;
          }
        },
        'o' | 'A' => {
          // On state
          if amount == 0 {
            // Not preceded by a number
            pattern.cells.push((x, y));
            x += 1;
          } else {
            for i in 0..amount {
              pattern.cells.push((x + i, y));
            }
            x += amount;
            amount = 0;
          }
        },
        '0' => amount *= 10,
        '1' => amount = amount * 10 + 1,
        '2' => amount = amount * 10 + 2,
        '3' => amount = amount * 10 + 3,
        '4' => amount = amount * 10 + 4,
        '5' => amount = amount * 10 + 5,
        '6' => amount = amount * 10 + 6,
        '7' => amount = amount * 10 + 7,
        '8' => amount = amount * 10 + 8,
        '9' => amount = amount * 10 + 9,
        '!' => {
          // The end of this pattern was reached
          return Ok(pattern);
        },
        unknown => {
          return Err(color_eyre::eyre::eyre!(
            "Unexpected character `{}` while reading data from a `.rle` file.",
            unknown
          ))
        },
      }
    }
    if amount != 0 {
      y += amount;
    } else {
      y += 1;
    }
  }

  Ok(pattern)
}
