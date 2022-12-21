mod cards;

use std::io::Write;

fn new_game() -> cards::Game {
    let new_game = cards::Game::new();

    new_game
}

fn take_input(player: usize) -> String {
    let mut input = String::new();

    print!("P{}> ", player);
    std::io::stdout().flush().unwrap();

    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    input
}

fn make_turn(game: &mut cards::Game) {
    let board_deck = &mut game.deck;

    for i in 0..2 {
        let player_deck = &mut game.players[i].deck;

        let drawn_card = board_deck.cards.pop();

        player_deck.cards.push(drawn_card.unwrap());

        // Await player input
        take_input(i);
    }
}

fn main() {
    let mut game = new_game();

    println!("Board Deck has {} cards!", game.deck.cards.len());

    loop {
        for j in 0..2 {
            println!("P{} Deck: {}", j, game.players[j].deck);
        }
        make_turn(&mut game);
    }
}
