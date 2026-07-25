use std::fs;
use crate::tile::TileType;

pub const BLOCK_SIZE: usize = 32;

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
        '-' => {
          block.tiles[index] = TileType::Floor;
          index += 1;
        },
        'W' => {
          block.tiles[index] = TileType::Wall;
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

impl Block {
  pub fn xy_idx(&self, x: i32, y: i32) -> usize {
    (y as usize * BLOCK_SIZE) + x as usize
}
}