pub mod array;
pub mod engine;

use std::{collections::HashMap, io};

use rand::random;

fn main() {
    let iters = 1000000;

    let mut total_moves = 0;
    let mut max_reached = Vec::with_capacity(iters);
    let mut score_reached = Vec::with_capacity(iters);
    for i in 0..iters {
        if i % 200 == 0 {
            println!("Iter: {}", i);
        }

        let b = run_basic_strat();
        max_reached.push(b.get_max());
        score_reached.push(b.get_score());
        total_moves += b.num_moves();
    }

    println!("Total Moves: {}", total_moves);

    let mr = array::Tensor::new(max_reached);
    let sr = array::Tensor::new(score_reached);

    println!("{:?}", mr.unique());

    // start_cli();
}

fn run_basic_strat() -> engine::Board {
    let mut b = engine::Board::new();

    let mut board_changed: bool;
    let mut game_over = false;
    // let (board_changed, game_over) = (true, false);

    while !game_over {
        let move_ = random::<bool>() as usize;

        (board_changed, game_over) = b.make_move([engine::Move::Down, engine::Move::Right][move_]);

        // println!(b);

        if !board_changed && !game_over {
            (board_changed, game_over) =
                b.make_move([engine::Move::Down, engine::Move::Right][(move_ + 1) % 2]);

            if !board_changed && !game_over {
                (board_changed, game_over) = b.make_move(engine::Move::Up);

                if !board_changed && !game_over {
                    (_, game_over) = b.make_move(engine::Move::Left);
                }
            }
        }
    }

    return b;
}

fn start_cli() {
    let mut b = engine::Board::new();

    println!("{}", b);

    let mut input = String::new();

    let mut map: HashMap<&str, engine::Move> = HashMap::new();
    map.insert("w", engine::Move::Up);
    map.insert("a", engine::Move::Left);
    map.insert("s", engine::Move::Down);
    map.insert("d", engine::Move::Right);

    loop {
        io::stdin()
            .read_line(&mut input)
            .expect("failed to readline");
        // get first value of string
        if map.contains_key(input.trim()) {
            let dir = map[input.trim()];

            let (_, game_over) = b.make_move(dir);

            // clear the terminal screen
            print!("{}[2J", 27 as char);

            // println!("Moves made: ");
            println!("{}", b);

            if game_over {
                println!("Game is over!");
                break;
            }
        }
        input.clear();
    }
}
