const GRID_SIZE: usize = 8;

struct GameState {
    state: [bool; GRID_SIZE * GRID_SIZE],
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            state: [false; GRID_SIZE * GRID_SIZE],
        }
    }

    pub fn from_random() -> GameState {
        let mut new_game = GameState::new();
        for field in &mut new_game.state {
            *field = rand::random();
        }
        new_game
    }

    pub fn print(&self) {
        let mut i: usize = 0;
        let len = self.state.len();

        while i < len {
            if self.state[i] {
                print!("1");
            } else {
                print! {"0"};
            }
            if (i + 1) % (GRID_SIZE) == 0 {
                print!("\n");
            }
            i += 1;
        }
    }
}

fn main() {
    let state = GameState::from_random();
    state.print();
}
