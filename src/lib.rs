#![feature(conservative_impl_trait)]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate rand;
#[macro_use]
extern crate serde_derive;

extern crate brdgme_color;
extern crate brdgme_game;
extern crate brdgme_markup;

use rand::Rng;

use brdgme_game::{CommandResponse, Gamer, Log, Status};
use brdgme_game::game::gen_placings;
use brdgme_game::errors::*;
use brdgme_game::command::Spec as CommandSpec;
use brdgme_markup::Node as N;

pub mod board;
pub mod tile;
pub mod casino;
pub mod render;
pub mod card;
mod command;

use board::{Board, BoardTile, Loc};
use casino::Casino;
use tile::TILES;
use card::{render_cards, shuffled_deck, Card};
use render::render_cash;
use command::Command;

pub const STARTING_CARDS: usize = 2;
pub const PLAYER_DICE: usize = 12;
pub const PLAYER_OWNER_TOKENS: usize = 10;
pub const CASINO_CARDS: usize = 9;
pub const CASINO_TILES: usize = 9;

pub static POINT_STOPS: &'static [usize] = &[
    0,
    1,
    2,
    3,
    4,
    5,
    6,
    7,
    8,
    10,
    12,
    14,
    16,
    18,
    20,
    23,
    26,
    29,
    32,
    36,
    40,
    44,
    49,
    54,
    60,
    66,
    73,
    81,
    90,
];

#[derive(Serialize, Deserialize)]
pub struct PubState {
    pub players: Vec<Player>,
    pub current_player: usize,
    pub remaining_deck: usize,
    pub played: Vec<Card>,
    pub board: Board,
    pub finished: bool,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerState {
    pub player: usize,
    pub state: Option<Player>,
    pub pub_state: PubState,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Player {
    pub cash: usize,
    pub points: usize,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Game {
    pub players: Vec<Player>,
    pub current_player: usize,
    pub deck: Vec<Card>,
    pub played: Vec<Card>,
    pub board: Board,
    pub finished: bool,
}

impl Gamer for Game {
    type PubState = PubState;
    type PlayerState = PlayerState;

    fn new(players: usize) -> Result<(Self, Vec<Log>)> {
        if players < 2 || players > 6 {
            bail!(ErrorKind::PlayerCount(2, 6, players));
        }
        let mut logs: Vec<Log> = vec![];
        let mut board = Board::default();
        let mut deck = shuffled_deck(players);
        let mut played: Vec<Card> = vec![];
        let current_player = rand::thread_rng().gen::<usize>() % players;
        let players: Vec<Player> = (0..players)
            .map(|p| {
                let cards: Vec<Card> = deck.drain(..STARTING_CARDS).collect();
                let cash = cards.iter().fold(0, |acc, c| match *c {
                    Card::Loc(l) => {
                        board.set(l, BoardTile::Owned { player: p });
                        acc + TILES[&l].starting_cash
                    }
                    Card::GameEnd => unreachable!(),
                });
                logs.push(Log::public(vec![
                    N::Player(p),
                    N::text(" drew "),
                    N::Group(render_cards(&cards)),
                    N::text(" and will start with "),
                    render_cash(cash),
                ]));
                let player = Player {
                    cash,
                    ..Player::default()
                };
                played.extend(cards);
                player
            })
            .collect();
        logs.push(Log::public(vec![
            N::Player(current_player),
            N::text(" will start the game"),
        ]));
        Ok((
            Game {
                players,
                current_player,
                board,
                deck,
                played,
                finished: false,
            },
            logs,
        ))
    }

    fn pub_state(&self) -> Self::PubState {
        PubState {
            players: self.players.clone(),
            current_player: self.current_player,
            remaining_deck: self.deck.len(),
            played: self.played.clone(),
            board: self.board.clone(),
            finished: self.finished,
        }
    }

    fn player_state(&self, player: usize) -> Self::PlayerState {
        PlayerState {
            player,
            state: self.players.get(player).cloned(),
            pub_state: self.pub_state(),
        }
    }

    fn command(
        &mut self,
        player: usize,
        input: &str,
        players: &[String],
    ) -> Result<CommandResponse> {
        let output = self.command_parser(player).parse(input, players)?;
        let (logs, can_undo) = match output.value {
            Command::Build { loc, casino } => self.build(player, &loc, &casino)?,
        };
        Ok(CommandResponse {
            logs,
            can_undo,
            remaining_input: output.remaining.to_string(),
        })
    }

    fn status(&self) -> Status {
        if self.finished {
            Status::Finished {
                placings: gen_placings(&self.players
                    .iter()
                    .map(|p| vec![p.points as i32, p.cash as i32])
                    .collect::<Vec<Vec<i32>>>()),
                stats: vec![],
            }
        } else {
            Status::Active {
                whose_turn: vec![self.current_player],
                eliminated: vec![],
            }
        }
    }

    fn command_spec(&self, _player: usize) -> Option<CommandSpec> {
        None
    }

    fn player_count(&self) -> usize {
        self.players.len()
    }

    fn player_counts() -> Vec<usize> {
        (2..7).collect()
    }
}

impl Game {
    fn can_build(&self, player: usize) -> bool {
        player == self.current_player
    }

    fn build(&mut self, p: usize, loc: &Loc, casino: &Casino) -> Result<(Vec<Log>, bool)> {
        if !TILES.contains_key(loc) {
            bail!(ErrorKind::InvalidInput("not a valid location".to_string()));
        }
        match self.board.get(loc) {
            BoardTile::Owned { player } if player == p => {}
            BoardTile::Built { .. } => bail!(ErrorKind::InvalidInput(
                "that location has already been built".to_string()
            )),
            _ => bail!(ErrorKind::InvalidInput(
                "you don't own that location".to_string()
            )),
        }
        if self.players[p].cash < TILES[loc].build_cost {
            bail!(ErrorKind::InvalidInput(
                "you don't have enough cash".to_string()
            ));
        }
        self.players[p].cash -= TILES[loc].build_cost;
        self.board.set(
            *loc,
            BoardTile::Built {
                casino: *casino,
                die: TILES[loc].die,
                player: p,
            },
        );
        Ok((vec![], true))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_counts_works() {
        assert_eq!(Game::player_counts(), vec![2, 3, 4, 5, 6]);
    }
}
