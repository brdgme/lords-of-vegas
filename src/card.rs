use rand::{self, Rng};

use brdgme_markup::Node as N;
use brdgme_color::*;

use std::fmt;

use board::{Block, Loc};
use tile::{Payout, TILES};
use STARTING_CARDS;
use casino::Casino;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Card {
    Loc(Loc),
    GameEnd,
}

pub fn shuffled_deck(players: usize) -> Vec<Card> {
    let mut rng = rand::thread_rng();
    let mut cards: Vec<Card> = TILES.keys().cloned().map(Card::Loc).collect();
    rng.shuffle(&mut cards);
    // Insert the game end card in the last quarter of the deck, taking into account the cards which
    // will be drawn by the players as adding the end card happens after players draw.
    let player_draw_count = players * STARTING_CARDS;
    let cards_len = cards.len();
    let quart_pile = (cards_len - player_draw_count) / 4;
    let quart_pos = rng.gen::<usize>() % quart_pile;
    cards.insert(cards_len - quart_pos, Card::GameEnd);
    cards
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Card::Loc(l) => write!(f, "{}{}", l.block, l.lot),
            Card::GameEnd => write!(f, "Game end"),
        }
    }
}

impl Card {
    pub fn render(&self) -> N {
        N::Bold(vec![N::text(format!("{}", self))])
    }
}

pub fn render_cards(cards: &[Card]) -> Vec<N> {
    let mut output: Vec<N> = vec![];
    for (i, c) in cards.iter().enumerate() {
        if i > 0 {
            output.push(N::text(" "));
        }
        output.push(c.render());
    }
    output
}

pub fn casino_card_count(cards: &[Card], casino: &Casino) -> usize {
    cards.iter().fold(0, |acc, c| match *c {
        Card::Loc(ref l) => match TILES[l].payout {
            Payout::Casino(c) if c == *casino => acc + 1,
            _ => acc,
        },
        _ => acc,
    })
}
