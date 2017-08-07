extern crate brdgme_cmd;
extern crate lords_of_vegas;

use lords_of_vegas::Game;
use brdgme_cmd::cli::cli;

use std::io;

fn main() {
    cli::<Game, _, _>(io::stdin(), &mut io::stdout());
}
