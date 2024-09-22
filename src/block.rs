use crate::tile::TileType;

pub const BLOCK_SIZE: usize = 32;

pub struct Block {
  pub tiles: Vec<TileType>
}

pub fn generate_blocks() -> Vec<Block> {
  let empty_floor = Block {
    tiles: vec![TileType::Floor; BLOCK_SIZE * BLOCK_SIZE]
  };

  let empty_ground = Block {
    tiles: vec![TileType::Ground; BLOCK_SIZE * BLOCK_SIZE]
  };

  vec!(empty_floor, empty_ground)
}

impl Block {
  pub fn xy_idx(&self, x: i32, y: i32) -> usize {
    (y as usize * BLOCK_SIZE) + x as usize
}
}