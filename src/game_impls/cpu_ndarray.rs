use crate::Gol;
use ndarray::prelude::*;
use ndarray::Array;

pub struct GameState {
    state: Array<bool, Ix2>,
    neighbor_offsets: Array<usize, Dim<[usize; 2]>>,
    boundaries: Array<usize, Dim<[usize; 2]>>,
}

impl Gol for GameState {
    fn from_slice(size: usize, slice: &[bool]) -> GameState {
        debug_assert!(size * size == slice.len());
        let mut state: Array<bool, _> = Array::default((size, size).f());

        for (src, dest) in slice.iter().zip(state.iter_mut()) {
            *dest = *src;
        }

        GameState {
            state,
            neighbor_offsets: Self::gen_neighbor_offsets(size, size),
            boundaries: Self::gen_boundary(size, size),
        }
    }

    fn to_vec(&self) -> Vec<bool> {
        let dim = self.state.dim();
        let mut vec = Vec::<bool>::with_capacity(dim.0 * dim.1);
        for field in self.state.iter() {
            vec.push(*field);
        }
        vec
    }

    fn to_next(&self) -> Box<dyn Gol> {
        let size = self.state.dim().0;

        let mut next = GameState {
            state: Array::<bool, _>::default((size, size).f()),
            neighbor_offsets: self.neighbor_offsets.to_owned(),
            boundaries: self.boundaries.to_owned(),
        };

        for (prev, next) in self.state.indexed_iter().zip(next.state.iter_mut()) {
            *next = self.next_state_for(prev.0);
        }

        Box::new(next)
    }

    fn print(&self) {
        for (i, is_alive) in self.state.indexed_iter() {
            print!("{}", if *is_alive { "1" } else { "0" });
            if i.1 == self.state.dim().1 - 1 {
                println!();
            }
        }
    }
}

impl GameState {
    fn new(size: usize) -> Self {
        GameState {
            state: Array::<bool, _>::default((size, size).f()),
            neighbor_offsets: Self::gen_neighbor_offsets(size, size),
            boundaries: Self::gen_boundary(size, size),
        }
    }
    pub fn from_random(size: usize) -> Box<dyn Gol> {
        let mut new_game = GameState::new(size);
        for field in &mut new_game.state {
            *field = rand::random();
        }
        Box::new(new_game)
    }

    pub fn next_state_for(&self, coords: (usize, usize)) -> bool {
        let mut total = 0;

        // This is ugly but turned out to be quite a lot faster than using a closure.
        let mut neighbors = array![
            [coords.0, coords.1],
            [coords.0, coords.1],
            [coords.0, coords.1],
            [coords.0, coords.1],
            [coords.0, coords.1],
            [coords.0, coords.1],
            [coords.0, coords.1],
            [coords.0, coords.1],
        ];

        neighbors += &self.neighbor_offsets;
        neighbors %= &self.boundaries;

        for neighbor_abs in neighbors.rows() {
            total += if self.state[(neighbor_abs[0], neighbor_abs[1])] {
                1
            } else {
                0
            };
        }

        // rules differ if the current cell is live or not
        if self.state[(coords.0, coords.1)] {
            total == 2 || total == 3
        } else {
            total == 3
        }
    }

    fn gen_neighbor_offsets(x_size: usize, y_size: usize) -> Array<usize, Dim<[usize; 2]>> {
        array![
            [x_size - 1, y_size - 1],
            [x_size - 1, y_size],
            [x_size - 1, y_size + 1],
            [x_size, y_size - 1],
            // skip self [0, 0]
            [x_size, y_size + 1],
            [x_size + 1, y_size - 1],
            [x_size + 1, y_size],
            [x_size + 1, y_size + 1],
        ]
    }

    fn gen_boundary(x_size: usize, y_size: usize) -> Array<usize, Dim<[usize; 2]>> {
        array![
            [x_size, y_size],
            [x_size, y_size],
            [x_size, y_size],
            [x_size, y_size],
            // skip self [0, 0]
            [x_size, y_size],
            [x_size, y_size],
            [x_size, y_size],
            [x_size, y_size],
        ]
    }
}
