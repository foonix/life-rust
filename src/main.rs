use game_impls::cpu;
use std::env;
use std::io::{self, Write};
use std::{thread, time};

use life_rust::game_impls;
use life_rust::Gol;

fn main() {
    let args: Vec<String> = env::args().collect();
    dbg!(&args);
    let mut size: usize = 16;
    if args.len() > 1 {
        let parsed = args[1].parse();
        match parsed {
            Ok(parsed_size) => size = parsed_size,
            _ => panic!(
                "Game size parameter ({}) must be positive intiger.",
                args[2]
            ),
        }
    }

    let mut back = cpu::GameState::from_random(size);
    let mut front: cpu::GameState;
    back.print();
    println!("------ <start>");
    for i in 0..32 {
        front = cpu::GameState::from_previous(&back);
        front.print();
        println!("------ {}", i);
        io::stdout().flush().unwrap();

        let ten_millis = time::Duration::from_millis(100);
        thread::sleep(ten_millis);

        back = front;
    }
}
