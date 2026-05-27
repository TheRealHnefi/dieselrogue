use std::fs;
use rltk::RandomNumberGenerator;
use crate::components::Direction;
use crate::tile::TileType;

pub const BLOCK_SIZE: usize = 32;

#[derive(Clone)]
pub struct Block {
  pub tiles: Vec<TileType>
}

fn generate_blocks(filter: &str) -> Vec<Block> {
  let mut blocks = vec!();

  let paths = fs::read_dir("resources/blocks").unwrap();
  for path in paths {
    let filename = path.as_ref().unwrap().file_name().into_string().unwrap();
    if filename.contains(filter) {
      let mut block = Block {
        tiles: vec![TileType::Ground; BLOCK_SIZE * BLOCK_SIZE]
      };

      let block_data = fs::read(path.unwrap().path()).unwrap();

      let mut index = 0;
      for character in block_data {
        match character as char {
          '.' => {
            block.tiles[index] = TileType::Ground;
            index += 1;
          },
          '_' => {
            block.tiles[index] = TileType::Road;
            index += 1;
          },
          '-' => {
            block.tiles[index] = TileType::Floor;
            index += 1;
          },
          'W' => {
            block.tiles[index] = TileType::Wall;
            index += 1;
          },
          'D' => {
            block.tiles[index] = TileType::Doorway;
            index += 1;
          },
          ' ' => {
            block.tiles[index] = TileType::Doorway;
            index += 1;
          },
          '#' => {
            block.tiles[index] = TileType::Fence;
            index += 1;
          },
          _ => ()
        }
      }
      blocks.push(block);
    }
  }

  blocks
}

pub fn generate_block_grid(size: usize) -> Option<Vec<Block>> {
  println!("Generating blocks");
  let mut rng = RandomNumberGenerator::new();
  let base_blocks = generate_blocks("road");
  let edge_blocks = generate_blocks("edge");
  let corner_blocks = generate_blocks("corner");
  let mut grid: Vec<Option<Block>> = vec![None; size * size];

  for x in 0 .. size {
    for y in 0 .. size {
      let index = grid_xy_idx(x, y, size);
      let above = if y > 0 { Some(grid_xy_idx(x, y - 1, size)) } else { None };
      //let below = if y < size - 1 { Some(grid_xy_idx(x, y + 1, size)) } else { None };
      let left = if x > 0 { Some(grid_xy_idx(x - 1, y, size)) } else { None };
      //let right = if x < size - 1 { Some(grid_xy_idx(x + 1, y, size)) } else { None };
    
      let block_above = match above {
        Some(idx) => Some(grid[idx].as_ref().unwrap()),
        None => None
      };
      let block_left = match left {
        Some(idx) => Some(grid[idx].as_ref().unwrap()),
        None => None
      };

      let active_blocks;
      if (x == 0 && y == 0)
        || (x == size - 1 && y == 0)
        || (x == size - 1 && y == size - 1)
        || (x == 0 && y == size - 1) {
          active_blocks = &corner_blocks;
        }
      else if x == 0 || x == size - 1 || y == 0 || y == size - 1 {
        active_blocks = &edge_blocks;
      }
      else {
        active_blocks = &base_blocks;
      }

      let mut valid_blocks = vec!();
      for block in active_blocks {
        if is_block_valid(block_left, Direction::Left, Some(&block))
          && is_block_valid(block_above, Direction::Up, Some(&block)) {
            let mut valid = true;
            if x == size - 1 {
              valid = is_block_valid(None, Direction::Right, Some(&block))
            }
            if valid && y == size - 1 {
              valid = is_block_valid(None, Direction::Down, Some(&block))
            }
            if valid {
              valid_blocks.push(block);
            }
          }
      }

      if valid_blocks.len() == 0 {
        println!("No valid blocks found for {} {}", x, y);
        //return None;
      }
      else {
        grid[index] = Some(valid_blocks[rng.range(0, valid_blocks.len())].clone());
      }
    }
  }

  let mut return_grid = vec!();
  for block in grid {
    match block {
      Some(block) => return_grid.push(block),
      None => return_grid.push(Block {tiles: vec![TileType::Ground; BLOCK_SIZE * BLOCK_SIZE]})
    }
  }
  return Some(return_grid);
}

fn grid_xy_idx(x: usize, y: usize, grid_size: usize) -> usize {
  (y * grid_size) + x
}

// Determine if placing block_1 next to block_2 would be valid given the tiles at the block edges
// Direction is "block 1 to the direction of block 2"
fn is_block_valid(block_1: Option<&Block>, direction: Direction, block_2: Option<&Block>) -> bool {
  for i in 0 .. BLOCK_SIZE {
    let (tile1_idx, tile2_idx) = match direction {
      Direction::Left =>  (block_xy_idx(BLOCK_SIZE - 1, i), block_xy_idx(0, i)),
      Direction::Right => (block_xy_idx(0, i),              block_xy_idx(BLOCK_SIZE - 1, i)),
      Direction::Up =>    (block_xy_idx(i, BLOCK_SIZE - 1), block_xy_idx(i, 0)),
      Direction::Down =>  (block_xy_idx(i, 0),              block_xy_idx(i, BLOCK_SIZE - 1)),
      _ => panic!("Invalid direction for block comparison")
    };

    let tile_1 = match block_1 {
      Some(block) => Some(block.tiles[tile1_idx]),
      None => None
    };
    let tile_2 = match block_2 {
      Some(block) => Some(block.tiles[tile2_idx]),
      None => None
    };
    if !valid_tile_neighbors(tile_1, tile_2) {
      return false;
    }
  }
  return true;
}

// Check if two tiles are valid to be neighbors across blocks. None represents the map edge.
fn valid_tile_neighbors(tile_1: Option<TileType>, tile_2: Option<TileType>) -> bool {
  fn matcher(t1: Option<TileType>, t2: Option<TileType>) -> bool {
    match (t1, t2) {
      (None, Some(TileType::Wall)) => true,
      (None, Some(TileType::Doorway)) => true,
      (Some(TileType::Wall), Some(TileType::Wall)) => true,
      (Some(TileType::Wall), Some(TileType::Floor)) => true,
      (Some(TileType::Wall), Some(TileType::Fence)) => true,
      (Some(TileType::Floor), Some(TileType::Floor)) => true,
      (Some(TileType::Floor), Some(TileType::Doorway)) => true,
      (Some(TileType::Ground), Some(TileType::Ground)) => true,
      (Some(TileType::Ground), Some(TileType::Doorway)) => true,
      (Some(TileType::Road), Some(TileType::Road)) => true,
      (Some(TileType::Road), Some(TileType::Doorway)) => true,
      (Some(TileType::Fence), Some(TileType::Fence)) => true,
      (_, _) => false
    }
  }
  // matcher does not check both ways, so needs to be called twice.
  return matcher(tile_1, tile_2) || matcher(tile_2, tile_1);
}

pub fn block_xy_idx(x: usize, y: usize) -> usize {
  (y as usize * BLOCK_SIZE) + x as usize
}

#[cfg(test)]
mod tests {
  use std::fs;
  use super::BLOCK_SIZE;

  fn is_tile_char(c: char) -> bool {
    matches!(c, '.' | '_' | '-' | 'W' | 'D' | ' ')
  }

  #[test]
  fn block_files_are_valid() {
    let dir = fs::read_dir("resources/blocks")
      .expect("resources/blocks directory must exist");

    let mut failures: Vec<String> = vec![];
    let mut file_count = 0;

    for entry in dir {
      let entry = entry.expect("directory entry must be readable");
      let filename = entry.file_name().into_string()
        .expect("filename must be valid UTF-8");
      if !filename.ends_with(".txt") {
        continue;
      }
      file_count += 1;

      let data = fs::read(entry.path())
        .unwrap_or_else(|_| panic!("block file '{}' must be readable", filename));

      let unknown: Vec<char> = data.iter()
        .map(|&b| b as char)
        .filter(|&c| !is_tile_char(c) && !c.is_ascii_whitespace())
        .collect();
      if !unknown.is_empty() {
        failures.push(format!(
          "  {}: unrecognized characters {:?} (silently dropped by parser)",
          filename, unknown
        ));
      }

      let tile_count = data.iter().filter(|&&b| is_tile_char(b as char)).count();
      let expected = BLOCK_SIZE * BLOCK_SIZE;
      if tile_count != expected {
        failures.push(format!(
          "  {}: {} tiles (expected {})",
          filename, tile_count, expected
        ));
      }

      let lines: Vec<&[u8]> = data.split(|&b| b == b'\n')
        .map(|l| l.strip_suffix(b"\r").unwrap_or(l))
        .filter(|l| !l.is_empty())
        .collect();
      if lines.len() != BLOCK_SIZE {
        failures.push(format!(
          "  {}: {} lines (expected {})",
          filename, lines.len(), BLOCK_SIZE
        ));
      }
      let bad_lines: Vec<String> = lines.iter().enumerate()
        .filter_map(|(i, line)| {
          if line.len() != BLOCK_SIZE {
            Some(format!("line {} is {} chars", i + 1, line.len()))
          } else {
            None
          }
        })
        .collect();
      if !bad_lines.is_empty() {
        failures.push(format!(
          "  {}: wrong line lengths: {}",
          filename, bad_lines.join(", ")
        ));
      }
    }

    assert!(file_count > 0, "no .txt files found in resources/blocks");
    assert!(
      failures.is_empty(),
      "{} block file(s) failed validation:\n{}",
      failures.len(),
      failures.join("\n")
    );
  }
}