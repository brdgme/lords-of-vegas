use brdgme_game::Log;
use brdgme_markup::Node as N;

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::iter::FromIterator;

use casino::Casino;
use tile::TILES;

use roll;

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

    pub fn render(&self) -> N {
        N::Bold(vec![N::text(format!("{}", self))])
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
        for bt in self.0.values() {
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

    pub fn casino_at(&self, loc: &Loc) -> Option<BoardCasino> {
        let casino = match self.get(loc) {
            BoardTile::Built { casino, .. } => casino,
            _ => return None,
        };

        let mut queue: HashSet<Loc> = HashSet::new();
        queue.insert(*loc);
        let mut visited: HashSet<Loc> = HashSet::new();
        let mut tiles: Vec<CasinoTile> = vec![];

        while !queue.is_empty() {
            let next = *queue.iter().next().expect("queue shouldn't be empty");
            visited.insert(next);
            queue.remove(&next);
            match self.get(&next) {
                BoardTile::Built {
                    casino: c,
                    player,
                    die,
                } if c == casino =>
                {
                    tiles.push(CasinoTile {
                        loc: next,
                        player,
                        die,
                    });
                    for n in next.neighbours() {
                        if !visited.contains(&n) {
                            queue.insert(n);
                        }
                    }
                }
                _ => {}
            }
        }

        Some(BoardCasino { casino, tiles })
    }

    pub fn casinos(&self) -> Vec<BoardCasino> {
        let mut visited: HashSet<Loc> = HashSet::new();
        let mut casinos: Vec<BoardCasino> = vec![];
        for loc in TILES.keys() {
            if visited.contains(loc) {
                continue;
            }
            match self.casino_at(loc) {
                Some(bc) => {
                    visited.extend(bc.tiles.iter().map(|t| t.loc));
                    casinos.push(bc);
                }
                None => {}
            }
        }
        casinos
    }

    pub fn reroll_at(&mut self, loc: &Loc) -> Option<usize> {
        let t = self.get(loc);
        match t {
            BoardTile::Built { casino, player, .. } => {
                let die = roll();
                self.set(
                    *loc,
                    BoardTile::Built {
                        casino,
                        player,
                        die,
                    },
                );
                Some(die)
            }
            _ => None,
        }
    }

    pub fn resolve_boss_ties(&mut self) -> Option<Vec<Log>> {
        let mut boss_tie = false;
        let mut logs: Vec<Log> = vec![];

        for bc in self.casinos() {
            let boss_tiles = bc.boss_tiles();
            let bosses: HashSet<usize> = HashSet::from_iter(boss_tiles.iter().map(|bt| bt.player));
            if bosses.len() <= 1 {
                // There is no boss tie.
                continue;
            }
            boss_tie = true;
            for bt in &boss_tiles {
                self.reroll_at(&bt.loc);
            }
        }

        if boss_tie {
            // Do another pass, we may have created a new boss tie.
            match self.resolve_boss_ties() {
                Some(new_logs) => logs.extend(new_logs),
                None => {}
            }
            Some(logs)
        } else {
            None
        }
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct CasinoTile {
    pub loc: Loc,
    pub player: usize,
    pub die: usize,
}

#[derive(PartialEq, Debug)]
pub struct BoardCasino {
    pub casino: Casino,
    pub tiles: Vec<CasinoTile>,
}

impl BoardCasino {
    pub fn boss_tiles(&self) -> Vec<CasinoTile> {
        let mut highest: usize = 0;
        let mut bosses: Vec<CasinoTile> = vec![];
        for t in &self.tiles {
            if t.die > highest {
                highest = t.die;
                bosses = vec![];
            }
            if t.die == highest {
                bosses.push(*t);
            }
        }
        bosses
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

    #[test]
    fn test_board_casino_at_works() {
        let mut b = Board::default();
        assert_eq!(None, b.casino_at(&(Block::A, 1).into()));

        b.set((Block::A, 1).into(), BoardTile::Owned { player: 0 });
        assert_eq!(None, b.casino_at(&(Block::A, 1).into()));

        b.set(
            (Block::A, 1).into(),
            BoardTile::Built {
                casino: Casino::Albion,
                die: 3,
                player: 0,
            },
        );
        assert_eq!(
            Some(BoardCasino {
                casino: Casino::Albion,
                tiles: vec![
                    CasinoTile {
                        loc: (Block::A, 1).into(),
                        die: 3,
                        player: 0,
                    },
                ],
            }),
            b.casino_at(&(Block::A, 1).into())
        );
        assert_eq!(
            vec![
                CasinoTile {
                    loc: (Block::A, 1).into(),
                    die: 3,
                    player: 0,
                },
            ],
            b.casino_at(&(Block::A, 1).into()).unwrap().boss_tiles()
        );

        // Set a diagonal and make sure it doesn't get included.
        b.set(
            (Block::A, 5).into(),
            BoardTile::Built {
                casino: Casino::Albion,
                die: 5,
                player: 0,
            },
        );
        assert_eq!(
            Some(BoardCasino {
                casino: Casino::Albion,
                tiles: vec![
                    CasinoTile {
                        loc: (Block::A, 1).into(),
                        die: 3,
                        player: 0,
                    },
                ],
            }),
            b.casino_at(&(Block::A, 1).into())
        );

        // Join the diagonal in and make sure it is.
        b.set(
            (Block::A, 2).into(),
            BoardTile::Built {
                casino: Casino::Albion,
                die: 2,
                player: 1,
            },
        );
        assert_eq!(
            Some(BoardCasino {
                casino: Casino::Albion,
                tiles: vec![
                    CasinoTile {
                        loc: (Block::A, 1).into(),
                        die: 3,
                        player: 0,
                    },
                    CasinoTile {
                        loc: (Block::A, 2).into(),
                        die: 2,
                        player: 1,
                    },
                    CasinoTile {
                        loc: (Block::A, 5).into(),
                        die: 5,
                        player: 0,
                    },
                ],
            }),
            b.casino_at(&(Block::A, 1).into())
        );
        assert_eq!(
            vec![
                CasinoTile {
                    loc: (Block::A, 5).into(),
                    die: 5,
                    player: 0,
                },
            ],
            b.casino_at(&(Block::A, 1).into()).unwrap().boss_tiles()
        );
    }

    #[test]
    fn test_board_casinos_works() {
        let mut b = Board::default();
        assert_eq!(0, b.casinos().len());

        b.set((Block::A, 1).into(), BoardTile::Owned { player: 0 });
        assert_eq!(0, b.casinos().len());

        b.set(
            (Block::A, 1).into(),
            BoardTile::Built {
                casino: Casino::Albion,
                die: 3,
                player: 0,
            },
        );
        assert_eq!(1, b.casinos().len());

        // Set a diagonal and make sure it doesn't get included.
        b.set(
            (Block::A, 5).into(),
            BoardTile::Built {
                casino: Casino::Albion,
                die: 5,
                player: 0,
            },
        );
        assert_eq!(2, b.casinos().len());

        // Join the diagonal in and make sure it is.
        b.set(
            (Block::A, 2).into(),
            BoardTile::Built {
                casino: Casino::Albion,
                die: 2,
                player: 1,
            },
        );
        assert_eq!(1, b.casinos().len());
    }
}
