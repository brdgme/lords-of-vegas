extern crate brdgme_game;
extern crate brdgme_rand_bot;
extern crate lords_of_vegas;

use lords_of_vegas::Game;
use brdgme_rand_bot::fuzz;

use std::io::stdout;

fn main() {
    fuzz::<Game, _>(&mut stdout());
}
