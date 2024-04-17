pub struct GameState {
    game_size: usize,
    state: Vec<bool>,
}

impl GameState {
    pub fn new(size: usize) -> GameState {
        GameState {
            game_size: size,
            state: vec![false; size * size],
        }
    }

    pub fn from_random(size: usize) -> GameState {
        let mut new_game = GameState::new(size);
        for field in &mut new_game.state {
            *field = rand::random();
        }
        new_game
    }

    // For now, this is just a test harness.
    // Maybe save/load later.
    #[cfg(test)]
    pub fn from_vec(size: usize, vec: &Vec<bool>) -> GameState {
        debug_assert!(size * size == vec.len());
        GameState {
            game_size: size,
            state: vec.to_owned(),
        }
    }

    pub fn from_previous(previous: &GameState) -> GameState {
        let mut next = GameState::new(previous.game_size);
        let size_as_i32: i32 = TryInto::<i32>::try_into(previous.game_size).unwrap();

        for i in 0..next.game_size * next.game_size {
            let mut total = 0;
            let (this_x, this_y) = GameState::coords_from_index(&next, i);
            for neighbor_y in -1..=1 {
                for neighbor_x in -1..=1 {
                    if neighbor_x != 0 || neighbor_y != 0 {
                        let neighbor_x_abs = (this_x + neighbor_x).rem_euclid(size_as_i32) as usize;
                        let neighbor_y_abs = (this_y + neighbor_y).rem_euclid(size_as_i32) as usize;

                        let neighbor_idx_abs = neighbor_y_abs * next.game_size + neighbor_x_abs;
                        if previous.state[neighbor_idx_abs] {
                            total += 1;
                        }
                    }
                }
            }
            // rules differ if the current cell is live or not
            if previous.state[i] {
                next.state[i] = total == 2 || total == 3;
            } else {
                next.state[i] = total == 3;
            }
        }

        next
    }

    fn coords_from_index(&self, i: usize) -> (i32, i32) {
        debug_assert!(i < self.game_size * self.game_size);
        let x: i32 = (i % self.game_size).try_into().unwrap();
        let y: i32 = (i / self.game_size).try_into().unwrap();
        (x, y)
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
            if (i + 1) % (self.game_size) == 0 {
                println!();
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
        let state1 = GameState::from_random(32);
        let state2 = GameState::from_previous(&state1);
        state2.print();
    }

    #[test]
    fn zeros_stay_zeros() {
        let start = vec![
            false, false, false, false, //
            false, false, false, false, //
            false, false, false, false, //
            false, false, false, false,
        ];

        let state1 = GameState::from_vec(4, &start);
        let state2 = GameState::from_previous(&state1);

        assert!(state2.state == start);
    }

    #[test]
    fn structure_tub() {
        let start = vec![
            false, false, false, false, false, //
            false, false, true, false, false, //
            false, true, false, true, false, //
            false, false, true, false, false, //
            false, false, false, false, false,
        ];

        let state1 = GameState::from_vec(5, &start);
        state1.print();
        let state2 = GameState::from_previous(&state1);
        state2.print();

        assert!(state2.state == start);
    }

    #[test]
    fn structure_box() {
        let start = vec![
            false, false, false, false, //
            false, true, true, false, //
            false, true, true, false, //
            false, false, false, false,
        ];

        let state1 = GameState::from_vec(4, &start);
        state1.print();
        let state2 = GameState::from_previous(&state1);
        state2.print();

        assert!(state2.state == start);
    }

    // same as box test, but in the game corner to test wrapping behavior.
    #[test]
    fn structure_box_wrapped() {
        let start = vec![
            true, false, false, true, //
            false, false, false, false, //
            false, false, false, false, //
            true, false, false, true,
        ];

        let state1 = GameState::from_vec(4, &start);
        state1.print();
        let state2 = GameState::from_previous(&state1);
        state2.print();

        assert!(state2.state == start);
    }

    #[test]
    fn structure_blinker() {
        let start = vec![
            false, false, false, false, false, //
            false, false, true, false, false, //
            false, false, true, false, false, //
            false, false, true, false, false, //
            false, false, false, false, false,
        ];
        let expected_mid = vec![
            false, false, false, false, false, //
            false, false, false, false, false, //
            false, true, true, true, false, //
            false, false, false, false, false, //
            false, false, false, false, false,
        ];

        let state1 = GameState::from_vec(5, &start);
        state1.print();
        let state2 = GameState::from_previous(&state1);
        state2.print();
        let state3 = GameState::from_previous(&state2);
        state3.print();

        assert!(state2.state == expected_mid);
        // verify that it repeats in 2 cycles
        assert!(state3.state == start);
    }

    #[test]
    fn structure_beacon() {
        let start = vec![
            false, false, false, false, false, false, //
            false, true, true, false, false, false, //
            false, true, true, false, false, false, //
            false, false, false, true, true, false, //
            false, false, false, true, true, false, //
            false, false, false, false, false, false,
        ];
        // the middle two blink
        let expected_mid = vec![
            false, false, false, false, false, false, //
            false, true, true, false, false, false, //
            false, true, false, false, false, false, //
            false, false, false, false, true, false, //
            false, false, false, true, true, false, //
            false, false, false, false, false, false,
        ];

        let state1 = GameState::from_vec(6, &start);
        state1.print();
        let state2 = GameState::from_previous(&state1);
        state2.print();
        let state3 = GameState::from_previous(&state2);
        state3.print();

        assert!(state2.state == expected_mid);
        // verify that it repeats in 2 cycles
        assert!(state3.state == start);
    }
}
