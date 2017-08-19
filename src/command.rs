use brdgme_game::command::parser::*;

use Game;
use board::Loc;

pub enum Command {
    Build(Loc),
}

impl Game {
    pub fn command_parser(&self, player: usize) -> Option<Box<Parser<Command>>> {
        if player != self.current_player {
            return None;
        }
        let mut parsers: Vec<Box<Parser<Command>>> = vec![];
        parsers.push(Box::new(self.build_parser(player)));
        Some(Box::new(OneOf::new(parsers)))
    }

    pub fn build_parser(&self, player: usize) -> impl Parser<Command> {
        Map::new(
            Chain2::new(
                Doc::name_desc("build", "build a casino at a location", Token::new("build")),
                AfterSpace::new(Doc::name_desc(
                    "loc",
                    "the location to build at",
                    loc_parser(self.board.player_locs(player)),
                )),
            ),
            |(_, l)| Command::Build(l),
        )
    }
}

fn loc_parser(mut locs: Vec<Loc>) -> impl Parser<Loc> {
    locs.sort();
    Enum::exact(locs)
}
