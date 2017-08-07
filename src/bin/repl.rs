extern crate brdgme_cmd;
extern crate lords_of_vegas;

use lords_of_vegas::Game;
use brdgme_cmd::repl;

fn main() {
    repl::<Game>();
}
