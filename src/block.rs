use std::fs;
use rltk::RandomNumberGenerator;
use crate::components::Direction;
use crate::tile::TileType;

pub const BLOCK_SIZE: usize = 32;

#[derive(Clone)]
pub struct Block {
  pub tiles: Vec<TileType>
}

pub fn generate_blocks() -> Vec<Block> {
  let mut blocks = vec!();

  let paths = fs::read_dir("resources/blocks").unwrap();
  for path in paths {
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
        _ => ()
      }
    }
    blocks.push(block);
  }

  blocks
}

pub fn generate_block_grid(size: usize) -> Option<Vec<Block>> {
  println!("Generating blocks");
  let mut rng = RandomNumberGenerator::new();
  let base_blocks = generate_blocks();
  let mut grid: Vec<Option<Block>> = vec![None; size * size];

  for y in 0 .. size {
    for x in 0 .. size {
      let index = grid_xy_idx(x, y, size);
      let above = if y > 0 { Some(grid_xy_idx(x, y - 1, size)) } else { None };
      let below = if y < size - 1 { Some(grid_xy_idx(x, y + 1, size)) } else { None };
      let left = if x > 0 { Some(grid_xy_idx(x - 1, y, size)) } else { None };
      let right = if x < size - 1 { Some(grid_xy_idx(x + 1, y, size)) } else { None };
    
      let block_above = match above {
        Some(idx) => Some(grid[idx].as_ref().unwrap()),
        None => None
      };
      let block_left = match left {
        Some(idx) => Some(grid[idx].as_ref().unwrap()),
        None => None
      };

      let mut valid_blocks = vec!();
      for block in &base_blocks {
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
        return None;
      }

      grid[index] = Some(valid_blocks[rng.range(0, valid_blocks.len())].clone());
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
// Does not check both ways, so needs to be called twice.
fn valid_tile_neighbors(tile_1: Option<TileType>, tile_2: Option<TileType>) -> bool {
  fn matcher(t1: Option<TileType>, t2: Option<TileType>) -> bool {
    match (t1, t2) {
      (None, Some(TileType::Wall)) => true,
      (None, Some(TileType::Doorway)) => true,
      (Some(TileType::Wall), Some(TileType::Ground)) => true,
      (Some(TileType::Wall), Some(TileType::Road)) => true,
      (Some(TileType::Wall), Some(TileType::Floor)) => true,
      (Some(TileType::Wall), Some(TileType::Wall)) => true,
      (Some(TileType::Floor), Some(TileType::Floor)) => true,
      (Some(TileType::Floor), Some(TileType::Doorway)) => true,
      (Some(TileType::Ground), Some(TileType::Ground)) => true,
      (Some(TileType::Ground), Some(TileType::Doorway)) => true,
      (Some(TileType::Road), Some(TileType::Road)) => true,
      (Some(TileType::Road), Some(TileType::Doorway)) => true,
      (_, _) => false
    }
  }
  return matcher(tile_1, tile_2) || matcher(tile_2, tile_1);
}

pub fn block_xy_idx(x: usize, y: usize) -> usize {
  (y as usize * BLOCK_SIZE) + x as usize
}