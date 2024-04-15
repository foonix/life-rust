const GRID_SIZE: usize = 16;

pub struct GameState {
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

    pub fn from_previous(previous: &GameState) -> GameState {
        let mut next = GameState::new();

        let mut i = 0;
        while i < GRID_SIZE * GRID_SIZE {
            let mut total = 0;
            let (x, y) = GameState::coords_from_index(i);

            // top row
            if previous.state[GameState::index_from_coords(x - 1, y - 1)] {
                total += 1
            };
            if previous.state[GameState::index_from_coords(x, y - 1)] {
                total += 1
            };
            if previous.state[GameState::index_from_coords(x + 1, y - 1)] {
                total += 1
            };
            // left/right
            if previous.state[GameState::index_from_coords(x - 1, y)] {
                total += 1
            };
            if previous.state[GameState::index_from_coords(x + 1, y)] {
                total += 1
            };
            // bottom row
            if previous.state[GameState::index_from_coords(x - 1, y + 1)] {
                total += 1
            };
            if previous.state[GameState::index_from_coords(x, y + 1)] {
                total += 1
            };
            if previous.state[GameState::index_from_coords(x + 1, y + 1)] {
                total += 1
            };

            next.state[i] = total == 2 || total == 3;

            i += 1;
        }

        next
    }

    fn coords_from_index(i: usize) -> (i32, i32) {
        assert!(i < GRID_SIZE * GRID_SIZE);
        let x: i32 = (i % GRID_SIZE).try_into().unwrap();
        let y: i32 = (i / GRID_SIZE).try_into().unwrap();
        (x, y)
    }

    fn index_from_coords(x: i32, y: i32) -> usize {
        let size_as_i32: i32 = TryInto::<i32>::try_into(GRID_SIZE).unwrap();

        // wrap grid edge
        let wrapped_x = x.rem_euclid(size_as_i32);
        let wrapped_y = y.rem_euclid(size_as_i32);

        assert!(wrapped_x >= 0);
        assert!(wrapped_y >= 0);

        let idx = wrapped_y * size_as_i32 + wrapped_x;
        idx.try_into().unwrap()
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

#[cfg(test)]
mod tests {
    use super::GameState;

    #[test]
    fn smoke() {
        let state1 = GameState::from_random();
        let state2 = GameState::from_previous(&state1);
        state2.print();
    }
}
