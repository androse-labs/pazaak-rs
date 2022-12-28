mod cards;
mod messages;
use cards::Deck;
use core::time;
use crossterm::style::Stylize;
use std::{env, fmt, io::Write, process, thread};

fn new_game(player_deck: &Deck, opponent_deck: &Deck) -> cards::Game {
    let mut new_game = cards::Game::new();
    new_game.players[0].deck = player_deck.clone();
    new_game.players[1].deck = opponent_deck.clone();

    // Shuffle the decks
    new_game.players[0].deck.shuffle();
    new_game.players[1].deck.shuffle();

    // Both players draw 5 cards from their player specific deck
    for _ in 0..5 {
        new_game.players[0]
            .hand
            .cards
            .push(new_game.players[0].deck.draw());
        new_game.players[1]
            .hand
            .cards
            .push(new_game.players[1].deck.draw());
    }

    new_game
}

enum Action {
    Draw,
    Stand,
    EndTurn,
    Play,
    TurnStart,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Action::Draw => write!(f, "Draw"),
            Action::Stand => write!(f, "Stand"),
            Action::Play => write!(f, "Play"),
            Action::TurnStart => write!(f, "Turn Start"),
            Action::EndTurn => write!(f, "End Turn"),
        }
    }
}

fn print_log(message: &str) {
    println!("{} {}", "~".dark_grey(), message.dark_grey());
}

fn get_action_message(player: usize, action: Action) -> String {
    let message = match action {
        Action::Draw => {
            format!("{} Draws...", format!("Player {}", player + 1))
        }
        Action::Stand => {
            format!("{} Stands...", format!("Player {}", player + 1))
        }
        Action::Play => {
            format!("{} Plays...", format!("Player {}", player + 1))
        }
        Action::TurnStart => {
            format!("Starting {}'s Turn...", format!("Player {}", player + 1))
        }
        Action::EndTurn => {
            format!("Ending {}'s Turn...", format!("Player {}", player + 1))
        }
    };

    message
}

fn print_action_log(player: usize, action: Action) {
    let message = get_action_message(player, action);
    print_log(&message);
}

fn player_number_to_identifier(player: usize) -> String {
    match player {
        0 => String::from("You"),
        1 => String::from("Opponent"),
        _ => format!("Player {}", player + 1),
    }
}

// Expecting a string of "draw", "Stand", or "play" if it isn't one of those then it will return an error
fn get_input(player: usize) -> Action {
    let mut input = String::new();

    let input_indicator = format!("{}", "(stand, play, end)");

    println!(
        "What would you like to do? {}",
        input_indicator.yellow().italic()
    );

    print!("{}> ", player_number_to_identifier(player));
    std::io::stdout().flush().unwrap();

    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    input = input.trim().to_string();

    match input.as_str() {
        "stand" => Action::Stand,
        "play" => Action::Play,
        "end" => Action::EndTurn,
        _ => {
            print_log(messages::INVALID_INPUT_MESSAGE);
            get_input(player)
        }
    }
}

fn print_board(players: &[cards::Player; 2], board: &[cards::Board; 2]) {
    // Show Board State
    println!("{}", "---------------------------".blue().bold());

    println!("Opponent Board: {}", board[1]);
    println!(
        "Opponent Hand: {}",
        players[1].hand.get_anonymous_hand_string()
    );

    println!("{}", "~~~~~~~~~~~~~~~~~~~~~~~~~~~".blue().bold());

    println!("Your Board: {}", board[0]);
    println!("Your Hand: {}", players[0].hand);

    println!("{}", "---------------------------".blue().bold());
}

fn make_turn(game: &mut cards::Game) {
    for i in 0..2 {
        print_action_log(i, Action::TurnStart);

        // Skip turn if player is standing
        if let cards::Status::Standing = game.players[i].status {
            print_action_log(i, Action::Stand);
            continue;
        }

        let board_deck = &mut game.deck;
        let drawn_card = board_deck.draw();
        let player_board = &mut game.board[i];
        player_board.cards.push(drawn_card);

        print_action_log(i, Action::Draw);

        let mut is_finished = false;
        let mut played_card = false;

        while !is_finished {
            print_board(&game.players, &game.board);
            // Await player input
            let result = get_input(i);
            (is_finished, played_card) = process_action(
                result,
                i,
                &mut game.players[i],
                &mut game.board[i],
                played_card,
            );
        }
    }
    game.turn = game.turn + 1;
}

// Show hand with indexes beside them
fn print_hand_with_indexes(hand: &cards::Hand) {
    for (i, card) in hand.cards.iter().enumerate() {
        println!("{}: {}", i, card);
    }
}

fn take_card_input(player: usize, hand: &cards::Hand) -> usize {
    let mut input = String::new();

    let input_indicator = format!("(0-{})", hand.cards.len() - 1);

    println!(
        "Which card would you like to play? {}",
        input_indicator.yellow().italic()
    );

    print_hand_with_indexes(hand);

    print!("{}> ", player_number_to_identifier(player));
    std::io::stdout().flush().unwrap();

    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    input = input.trim().to_string();

    let card_index = input.parse::<usize>().unwrap();

    if card_index > hand.cards.len() - 1 {
        print_log(messages::INVALID_INPUT_MESSAGE);
        take_card_input(player, hand)
    } else {
        card_index
    }
}

fn process_action(
    action: Action,
    player_number: usize,
    player: &mut cards::Player,
    player_board: &mut cards::Board,
    already_played: bool,
) -> (bool, bool) {
    match action {
        Action::Stand => {
            print_log(&get_action_message(player_number, action));
            player.status = cards::Status::Standing;
        }
        Action::Play => {
            if !already_played {
                print_log(&get_action_message(player_number, action));

                let card_index = take_card_input(player_number, &player.hand);

                let card = player.hand.cards.remove(card_index);

                player_board.cards.push(card);

                return (false, true);
            } else {
                print_log(messages::ALREADY_PLAYED_MESSAGE);
                return (false, true);
            }
        }
        Action::EndTurn => {
            print_log(&get_action_message(player_number, action));
        }
        _ => {
            print_log(&get_action_message(player_number, action));
        }
    }

    (true, false)
}

fn validate_deck_paths(paths: &[String]) {
    print_log("Validating Deck Paths...");
    for path in paths {
        if !std::path::Path::new(path).exists() {
            eprintln!("{} '{}'", messages::INVALID_DECK_PATH_MESSAGE, path);
            process::exit(1);
        }
        print_log(&format!("{} '{}'", "Found Deck Path:", path));
    }
    print_log("Deck Paths Validated!");
}

fn read_deck_file(path: &str) -> cards::Deck {
    let mut deck = cards::Deck::new();

    let file = std::fs::read_to_string(path).expect("Unable to read file");

    for line in file.lines() {
        let found_int = line.parse::<i8>();

        match found_int {
            Ok(value) => {
                deck.cards.push(cards::Card::new(value));
            }
            Err(_) => {
                eprintln!("{} '{}'", messages::INVALID_DECK_FILE_MESSAGE, path);
                process::exit(1);
            }
        }
    }

    deck
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!(
            "{} {}",
            messages::INVALID_ARGUMENTS_MESSAGE,
            messages::USAGE_MESSAGE
        );
        process::exit(1);
    }
    let player_deck_path = &args[1];
    let opponent_deck_path = &args[2];
    let deck_paths = vec![player_deck_path.to_string(), opponent_deck_path.to_string()];
    validate_deck_paths(&deck_paths);

    messages::print_welcome_message();

    let player_deck = &read_deck_file(player_deck_path);
    let opponent_deck = &read_deck_file(opponent_deck_path);

    let mut pzk_match = cards::Match::new();
    pzk_match.games.push(new_game(player_deck, opponent_deck));

    // Host Match
    while pzk_match.score[0] < 3 && pzk_match.score[1] < 3 {
        let current_game_index = pzk_match.round - 1;

        // Turn Logic
        loop {
            println!("{}", "===========================".blue());
            println!("{}", pzk_match);
            let current_game = &mut pzk_match.games[current_game_index];
            make_turn(current_game);
            if current_game.players[0].status == cards::Status::Standing
                && current_game.players[1].status == cards::Status::Standing
            {
                break;
            }
        }
        // Post Game Logic
        let winner = pzk_match.games[pzk_match.round - 1].check_win();
        match winner {
            Some(winner) => {
                println!("{} wins!", player_number_to_identifier(winner));
                pzk_match.score[winner] = pzk_match.score[winner] + 1;
            }
            None => println!("Draw!"),
        }

        // Wait 2000ms
        thread::sleep(time::Duration::from_millis(2000));

        pzk_match.round = pzk_match.round + 1;

        // Prepare next game
        print_log(messages::PREPARING_NEXT_GAME_MESSAGE);
        println!("{}", "===========================".blue().bold());
        pzk_match.games.push(new_game(player_deck, opponent_deck));
    }
}
