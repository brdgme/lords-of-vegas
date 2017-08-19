use brdgme_game::command::parser::*;

use Game;
use casino::{Casino, CASINOS};
use board::Loc;

pub enum Command {
    Build { loc: Loc, casino: Casino },
}

impl Game {
    pub fn command_parser(&self, player: usize) -> Box<Parser<Command>> {
        let mut parsers: Vec<Box<Parser<Command>>> = vec![];
        if self.can_build(player) {
            parsers.push(Box::new(self.build_parser(player)));
        }
        Box::new(OneOf::new(parsers))
    }

    pub fn build_parser(&self, player: usize) -> impl Parser<Command> {
        Map::new(
            Chain3::new(
                Doc::name_desc("build", "build a casino at a location", Token::new("build")),
                AfterSpace::new(Doc::name_desc(
                    "loc",
                    "the location to build at",
                    loc_parser(self.board.player_locs(player)),
                )),
                AfterSpace::new(Doc::name_desc(
                    "casino",
                    "the casino to build",
                    casino_parser(),
                )),
            ),
            |(_, loc, casino)| Command::Build { loc, casino },
        )
    }
}

fn loc_parser(mut locs: Vec<Loc>) -> impl Parser<Loc> {
    locs.sort();
    Enum::exact(locs)
}

fn casino_parser() -> impl Parser<Casino> {
    Enum::partial(CASINOS.to_owned())
}
