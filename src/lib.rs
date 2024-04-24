pub mod game_impls;

pub trait Gol {
    fn new(size: usize) -> Self;
    fn from_vec(ize: usize, vec: &Vec<bool>) -> Self;
    fn to_vec(&self) -> Vec<bool>;
    fn from_previous(previous: &Self) -> Self;
    fn print(&self);
}
