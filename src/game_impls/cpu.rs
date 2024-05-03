use crate::Gol;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

pub struct GameState {
    game_size: usize,
    state: Vec<bool>,
}

impl Gol for GameState {
    fn from_slice(size: usize, slice: &[bool]) -> GameState {
        debug_assert!(size * size == slice.len());
        GameState {
            game_size: size,
            state: slice.to_owned(),
        }
    }

    fn to_vec(&self) -> Vec<bool> {
        self.state.to_vec()
    }

    fn to_next(&self) -> Box<dyn Gol> {
        let mut next = GameState::new(self.game_size);
        let size_as_i32: i32 = TryInto::<i32>::try_into(self.game_size).unwrap();

        for (next, (i, _)) in next.state.iter_mut().zip(self.state.iter().enumerate()) {
            *next = self.next_state_for(i, size_as_i32);
        }

        Box::new(next)
    }

    fn print(&self) {
        for (i, is_alive) in self.state.iter().enumerate() {
            print!("{}", if *is_alive { "1" } else { "0" });
            if (i + 1) % (self.game_size) == 0 {
                println!();
            }
        }
    }
}

impl GameState {
    fn new(size: usize) -> Self {
        GameState {
            game_size: size,
            state: vec![false; size * size],
        }
    }
    pub fn from_random(size: usize) -> Box<dyn Gol> {
        let mut new_game = GameState::new(size);
        for field in &mut new_game.state {
            *field = rand::random();
        }
        Box::new(new_game)
    }

    pub fn from_previous_parallel(previous: &GameState, threads: usize) -> GameState {
        let next_state = vec![false; previous.game_size * previous.game_size];

        let previous_arc = Arc::new(previous);
        let next_arc = Arc::new(Mutex::new(next_state));

        let target_thread_slice_size = (previous.game_size * previous.game_size).div_ceil(threads);

        let scope_next_arc = next_arc.clone();
        let size_as_i32: i32 = TryInto::<i32>::try_into(previous.game_size).unwrap();

        thread::scope(move |s| {
            let mut next_thread_offset = 0;

            for _ in 0..threads {
                let thread_next_arc = scope_next_arc.clone();
                let thread_offset = next_thread_offset;

                let thread_slize_size =
                    if thread_offset + target_thread_slice_size < previous_arc.state.len() {
                        target_thread_slice_size
                    } else {
                        previous_arc.state.len() - thread_offset
                    };
                next_thread_offset += thread_slize_size;

                s.spawn(move || {
                    let start = thread_offset;
                    let end = thread_offset + thread_slize_size;
                    for i in start..end {
                        let alive = previous.next_state_for(i, size_as_i32);
                        {
                            let state = &mut thread_next_arc.lock().unwrap();
                            state[i] = alive;
                        }
                    }
                });
            }
        });

        GameState {
            game_size: previous.game_size,
            state: Arc::try_unwrap(next_arc).unwrap().into_inner().unwrap(),
        }
    }

    fn coords_from_index(&self, i: usize) -> (i32, i32) {
        debug_assert!(i < self.game_size * self.game_size);
        let x: i32 = (i % self.game_size).try_into().unwrap();
        let y: i32 = (i / self.game_size).try_into().unwrap();
        (x, y)
    }

    fn next_state_for(&self, i: usize, size_as_i32: i32) -> bool {
        let mut total = 0;
        let (this_x, this_y) = GameState::coords_from_index(self, i);
        for neighbor_y in -1..=1 {
            for neighbor_x in -1..=1 {
                if neighbor_x != 0 || neighbor_y != 0 {
                    let neighbor_x_abs = (this_x + neighbor_x).rem_euclid(size_as_i32) as usize;
                    let neighbor_y_abs = (this_y + neighbor_y).rem_euclid(size_as_i32) as usize;

                    let neighbor_idx_abs = neighbor_y_abs * self.game_size + neighbor_x_abs;
                    if self.state[neighbor_idx_abs] {
                        total += 1;
                    }
                }
            }
        }
        // rules differ if the current cell is live or not
        if self.state[i] {
            total == 2 || total == 3
        } else {
            total == 3
        }
    }
}
