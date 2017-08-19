use std::collections::HashMap;
use std::fmt;

use casino::Casino;

const BLOCK_WIDTH: usize = 3;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Block {
    A,
    B,
    C,
    D,
    E,
    F,
}

pub static BLOCKS: &'static [Block] = &[Block::A, Block::B, Block::C, Block::D, Block::E, Block::F];

impl Block {
    pub fn max_lot(&self) -> Lot {
        match *self {
            Block::A | Block::B | Block::E => 6,
            Block::C => 12,
            Block::D | Block::F => 9,
        }
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Block::A => "A",
                Block::B => "B",
                Block::C => "C",
                Block::D => "D",
                Block::E => "E",
                Block::F => "F",
            }
        )
    }
}

pub type Lot = usize;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize, Copy, Clone)]
pub struct Loc {
    pub block: Block,
    pub lot: Lot,
}

impl From<(Block, Lot)> for Loc {
    fn from((block, lot): (Block, Lot)) -> Self {
        Loc { block, lot }
    }
}

impl Loc {
    pub fn neighbours(&self) -> Vec<Loc> {
        let mut n: Vec<Loc> = vec![];
        if self.lot > BLOCK_WIDTH {
            n.push((self.block, self.lot - BLOCK_WIDTH).into());
        }
        if self.lot % BLOCK_WIDTH != 1 {
            n.push((self.block, self.lot - 1).into());
        }
        if self.lot % BLOCK_WIDTH != 0 {
            n.push((self.block, self.lot + 1).into());
        }
        if self.lot <= self.block.max_lot() - BLOCK_WIDTH {
            n.push((self.block, self.lot + BLOCK_WIDTH).into());
        }
        n
    }
}

impl fmt::Display for Loc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.block, self.lot)
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum BoardTile {
    Unowned,
    Owned { player: usize },
    Built {
        player: usize,
        casino: Casino,
        die: usize,
    },
}

impl Default for BoardTile {
    fn default() -> Self {
        BoardTile::Unowned
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Board(HashMap<Loc, BoardTile>);

#[derive(Default, Copy, Clone)]
pub struct UsedResources {
    pub dice: usize,
    pub tokens: usize,
}

impl Board {
    pub fn get(&self, loc: &Loc) -> BoardTile {
        self.0.get(loc).cloned().unwrap_or_default()
    }

    pub fn set(&mut self, loc: Loc, bt: BoardTile) {
        self.0.insert(loc, bt);
    }

    pub fn used_resources(&self, p: usize) -> UsedResources {
        let mut used = UsedResources::default();
        for (_, bt) in &self.0 {
            match *bt {
                BoardTile::Owned { player } if player == p => used.tokens += 1,
                BoardTile::Built { player, .. } if player == p => used.dice += 1,
                _ => {}
            }
        }
        used
    }

    pub fn casino_tile_count(&self, c: &Casino) -> usize {
        self.0.iter().fold(0, |acc, (_, bt)| match *bt {
            BoardTile::Built { casino, .. } if casino == *c => acc + 1,
            _ => acc,
        })
    }

    pub fn player_locs(&self, p: usize) -> Vec<Loc> {
        self.0
            .iter()
            .filter_map(|(l, bt)| match *bt {
                BoardTile::Owned { player } if player == p => Some(*l),
                _ => None,
            })
            .collect()
    }
}

impl Default for Board {
    fn default() -> Self {
        Board(HashMap::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_neighbours<I: Into<Loc>>(l: I, n: Vec<I>) {
        let mut expected = n.into_iter().map(|n| n.into()).collect::<Vec<Loc>>();
        expected.sort();
        let mut actual = l.into().neighbours();
        actual.sort();
        assert_eq!(expected, actual);
    }

    #[test]
    fn loc_neighbours_works() {
        use self::Block::*;

        assert_neighbours((A, 1), vec![(A, 2), (A, 4)]);
        assert_neighbours((A, 2), vec![(A, 1), (A, 3), (A, 5)]);
        assert_neighbours((A, 3), vec![(A, 2), (A, 6)]);
        assert_neighbours((A, 4), vec![(A, 1), (A, 5)]);
        assert_neighbours((A, 5), vec![(A, 2), (A, 4), (A, 6)]);
        assert_neighbours((A, 6), vec![(A, 3), (A, 5)]);
        assert_neighbours((C, 8), vec![(C, 5), (C, 7), (C, 9), (C, 11)]);
    }
}
