use game_impls::cpu;
use std::io::{self, Write};
use std::{thread, time};

mod game_impls;

fn main() {
    let mut back = cpu::GameState::from_random(16);
    let mut front: cpu::GameState;
    back.print();
    println!("------ <start>");
    let mut i = 0;
    while i < 32 {
        front = cpu::GameState::from_previous(&back);
        front.print();
        println!("------ {}", i);
        io::stdout().flush().unwrap();

        let ten_millis = time::Duration::from_millis(100);
        thread::sleep(ten_millis);

        back = front;
        i += 1;
    }
}
