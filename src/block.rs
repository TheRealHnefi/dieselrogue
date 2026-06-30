use std::fs;
use rltk::RandomNumberGenerator;
use crate::components::Direction;
use crate::tile::TileType;

pub const BLOCK_SIZE: usize = 32;

#[derive(Clone)]
pub struct Block {
  pub tiles: Vec<TileType>
}

fn rotate_block_90cw(block: &Block) -> Block {
  let n = BLOCK_SIZE;
  let mut tiles = vec![TileType::Ground; n * n];
  for y in 0..n {
    for x in 0..n {
      tiles[x * n + (n - 1 - y)] = block.tiles[y * n + x];
    }
  }
  Block { tiles }
}

fn mirror_horizontal(block: &Block) -> Block {
  let n = BLOCK_SIZE;
  let mut tiles = vec![TileType::Ground; n * n];
  for y in 0..n {
    for x in 0..n {
      tiles[y * n + (n - 1 - x)] = block.tiles[y * n + x];
    }
  }
  Block { tiles }
}

fn mirror_vertical(block: &Block) -> Block {
  let n = BLOCK_SIZE;
  let mut tiles = vec![TileType::Ground; n * n];
  for y in 0..n {
    for x in 0..n {
      tiles[(n - 1 - y) * n + x] = block.tiles[y * n + x];
    }
  }
  Block { tiles }
}

fn mirror_both(block: &Block) -> Block {
  let n = BLOCK_SIZE;
  let mut tiles = vec![TileType::Ground; n * n];
  for y in 0..n {
    for x in 0..n {
      tiles[(n - 1 - y) * n + (n - 1 - x)] = block.tiles[y * n + x];
    }
  }
  Block { tiles }
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
          'x' => {
            block.tiles[index] = TileType::Window;
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
      let r90  = rotate_block_90cw(&block);
      let r180 = rotate_block_90cw(&r90);
      let r270 = rotate_block_90cw(&r180);
      for base in [block, r90, r180, r270] {
        blocks.push(mirror_horizontal(&base));
        blocks.push(mirror_vertical(&base));
        blocks.push(mirror_both(&base));
        blocks.push(base);
      }
    }
  }

  blocks
}

pub fn generate_block_grid(size: usize, rng: &mut RandomNumberGenerator) -> Option<Vec<Block>> {
  println!("Generating blocks");

  let corner_blocks   = generate_blocks("corner");
  let edge_blocks     = generate_blocks("edge");
  let mut base_blocks = generate_blocks("road");
  base_blocks.extend(generate_blocks("building"));
  let middle_variants = generate_blocks("middleblock");

  let nc = corner_blocks.len();
  let ne = edge_blocks.len();

  // Pool layout: corners | edges | base | middle.
  // Middle blocks are excluded from regular interior cells and only placed at the center.
  let mut all_blocks: Vec<Block> = corner_blocks;
  all_blocks.extend(edge_blocks);
  all_blocks.extend(base_blocks);
  let middle_start = all_blocks.len();
  all_blocks.extend(middle_variants);
  let nb = all_blocks.len();
  let n_middle = nb - middle_start;

  // Pre-compute pairwise compatibility for all 4 directions.
  // compat[d][a * nb + b] = is_block_valid(all_blocks[a], dirs[d], all_blocks[b])
  // Directions indexed: 0=Left, 1=Right, 2=Up, 3=Down
  let dirs = [Direction::Left, Direction::Right, Direction::Up, Direction::Down];
  let compat: Vec<Vec<bool>> = dirs.iter().map(|&dir| {
    let mut t = vec![false; nb * nb];
    for a in 0..nb {
      for b in 0..nb {
        t[a * nb + b] = is_block_valid(Some(&all_blocks[a]), dir, Some(&all_blocks[b]));
      }
    }
    t
  }).collect();

  // Pre-compute map-edge compatibility.
  // edge_compat[d][b] = is_block_valid(None, dirs[d], all_blocks[b])
  let edge_compat: Vec<Vec<bool>> = dirs.iter().map(|&dir| {
    (0..nb).map(|b| is_block_valid(None, dir, Some(&all_blocks[b]))).collect()
  }).collect();

  // Inner ring parameters.
  let inner_margin = size / 4;
  let inner_min = inner_margin;
  let inner_max = size - 1 - inner_margin;
  let has_inner_ring = inner_margin > 0 && inner_min < inner_max;

  let n = size * size;

  // Initialise candidate sets: each cell holds the indices of all blocks it may use.
  let mut candidates: Vec<Vec<usize>> = (0..n).map(|idx| {
    let x = idx % size;
    let y = idx / size;

    let is_outer_corner = (x == 0 && y == 0) || (x == size-1 && y == 0)
                       || (x == size-1 && y == size-1) || (x == 0 && y == size-1);
    let is_outer_edge = !is_outer_corner && (x == 0 || x == size-1 || y == 0 || y == size-1);
    let on_inner_ring = has_inner_ring
      && x >= inner_min && x <= inner_max
      && y >= inner_min && y <= inner_max
      && (x == inner_min || x == inner_max || y == inner_min || y == inner_max);
    let is_inner_corner = on_inner_ring
      && (x == inner_min || x == inner_max) && (y == inner_min || y == inner_max);

    let mut pool: Vec<usize> = if is_outer_corner || is_inner_corner {
      (0..nc).collect()
    } else if is_outer_edge || on_inner_ring {
      (nc..nc + ne).collect()
    } else {
      (nc + ne..middle_start).collect()  // base only; middle blocks excluded
    };

    // Pre-filter for map boundary: the block's face on the map edge must be wall-like.
    // edge_compat[0] = Left, [1] = Right, [2] = Up, [3] = Down.
    if x == 0      { pool.retain(|&b| edge_compat[0][b]); }
    if x == size-1 { pool.retain(|&b| edge_compat[1][b]); }
    if y == 0      { pool.retain(|&b| edge_compat[2][b]); }
    if y == size-1 { pool.retain(|&b| edge_compat[3][b]); }

    pool
  }).collect();

  // Force the center cell to one of the loaded middle block variants.
  // Because middle blocks have few candidates relative to other interior cells,
  // WFC will collapse the center very early, seeding the rest of the layout.
  let center = (size / 2) * size + (size / 2);
  if n_middle > 0 {
    candidates[center] = (middle_start..nb).collect();
    println!("  Middle block variants available: {}", n_middle);
  }

  if candidates.iter().any(|c| c.is_empty()) {
    println!("Block files cannot satisfy boundary constraints — giving up.");
    return None;
  }

  // Seed propagation from every ring cell so interior candidates are pruned before WFC starts.
  // Include the center so its constraints radiate outward from the start.
  let mut seeds: Vec<usize> = (0..n).filter(|&idx| {
    let x = idx % size;
    let y = idx / size;
    let on_outer = x == 0 || x == size-1 || y == 0 || y == size-1;
    let on_inner = has_inner_ring
      && x >= inner_min && x <= inner_max
      && y >= inner_min && y <= inner_max
      && (x == inner_min || x == inner_max || y == inner_min || y == inner_max);
    on_outer || on_inner
  }).collect();
  if n_middle > 0 { seeds.push(center); }

  if wfc_propagate(&seeds, &mut candidates, size, nb, &compat).is_err() {
    println!("Initial propagation failed — block files may be incompatible.");
    return None;
  }

  // WFC with backtracking.
  wfc_solve(size, nb, &compat, candidates, rng)
    .map(|indices| indices.into_iter().map(|i| all_blocks[i].clone()).collect())
}

/// WFC main loop. Collapses minimum-entropy cells and backtracks on contradiction.
fn wfc_solve(
  size: usize,
  nb: usize,
  compat: &[Vec<bool>],
  mut candidates: Vec<Vec<usize>>,
  rng: &mut RandomNumberGenerator,
) -> Option<Vec<usize>> {
  let n = size * size;
  // History entry: (cell, block chosen, full candidates snapshot before collapse).
  let mut history: Vec<(usize, usize, Vec<Vec<usize>>)> = vec![];

  loop {
    // Find the uncollapsed cell with the fewest remaining candidates (min entropy).
    let min_cell = (0..n)
      .filter(|&i| candidates[i].len() > 1)
      .min_by_key(|&i| candidates[i].len());

    match min_cell {
      None => {
        // Every cell has exactly one candidate — done.
        return Some(candidates.into_iter().map(|c| c[0]).collect());
      }
      Some(cell) => {
        let pick = rng.range(0, candidates[cell].len());
        let chosen = candidates[cell][pick];

        history.push((cell, chosen, candidates.clone()));
        candidates[cell] = vec![chosen];

        if wfc_propagate(&[cell], &mut candidates, size, nb, compat).is_err() {
          // Contradiction — backtrack until we find a cell with untried candidates.
          loop {
            match history.pop() {
              None => return None, // exhausted all possibilities
              Some((bc, bb, mut saved)) => {
                saved[bc].retain(|&x| x != bb);
                if saved[bc].is_empty() {
                  continue; // this level is also exhausted, keep popping
                }
                candidates = saved;
                break;
              }
            }
          }
        }
      }
    }
  }
}

/// AC-3 constraint propagation. Removes candidates that have no compatible partner
/// in any adjacent cell. Propagates until stable or a contradiction is found.
/// `starts` is the set of cells whose candidates have just changed.
fn wfc_propagate(
  starts: &[usize],
  candidates: &mut Vec<Vec<usize>>,
  size: usize,
  nb: usize,
  compat: &[Vec<bool>],
) -> Result<(), ()> {
  let mut queue: Vec<usize> = starts.to_vec();
  let mut in_queue = vec![false; size * size];
  for &s in starts { in_queue[s] = true; }

  while let Some(cell) = queue.pop() {
    in_queue[cell] = false;
    let x = cell % size;
    let y = cell / size;

    // Each entry: (neighbour index, compat direction index).
    // The direction index encodes "cell is in that direction of the neighbour",
    // matching the compat table layout (0=Left,1=Right,2=Up,3=Down).
    let mut nbrs: Vec<(usize, usize)> = vec![];
    if x > 0      { nbrs.push((cell - 1,    1)); } // cell is RIGHT of left-nbr  → dir Right(1)
    if x < size-1 { nbrs.push((cell + 1,    0)); } // cell is LEFT  of right-nbr → dir Left(0)
    if y > 0      { nbrs.push((cell - size,  3)); } // cell is DOWN  of above-nbr → dir Down(3)
    if y < size-1 { nbrs.push((cell + size,  2)); } // cell is UP    of below-nbr → dir Up(2)

    let cell_cands = candidates[cell].clone();

    for (nbr, dir_idx) in nbrs {
      let prev_len = candidates[nbr].len();
      let row = &compat[dir_idx];

      // Keep only neighbour candidates that are compatible with at least one cell candidate.
      candidates[nbr].retain(|&b| {
        cell_cands.iter().any(|&a| row[a * nb + b])
      });

      if candidates[nbr].is_empty() {
        return Err(());
      }
      if candidates[nbr].len() < prev_len && !in_queue[nbr] {
        queue.push(nbr);
        in_queue[nbr] = true;
      }
    }
  }
  Ok(())
}

fn _grid_xy_idx(x: usize, y: usize, grid_size: usize) -> usize {
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
      (None, Some(TileType::Window)) => true,
      (Some(TileType::Wall), Some(TileType::Wall)) => true,
      (Some(TileType::Wall), Some(TileType::Floor)) => true,
      (Some(TileType::Wall), Some(TileType::Fence)) => true,
      (Some(TileType::Wall), Some(TileType::Ground)) => true,
      (Some(TileType::Floor), Some(TileType::Floor)) => true,
      (Some(TileType::Floor), Some(TileType::Doorway)) => true,
      (Some(TileType::Ground), Some(TileType::Ground)) => true,
      (Some(TileType::Ground), Some(TileType::Doorway)) => true,
      (Some(TileType::Road), Some(TileType::Road)) => true,
      (Some(TileType::Road), Some(TileType::Doorway)) => true,
      (Some(TileType::Fence), Some(TileType::Fence)) => true,
      (Some(TileType::Window), _) => true,
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
    matches!(c, '.' | '_' | '-' | 'W' | 'D' | ' ' | 'x' | '#')
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