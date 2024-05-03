pub mod compute;
pub mod cpu;
pub mod cpu_ndarray;

#[cfg(test)]
mod tests {
    use super::{compute, cpu, cpu_ndarray};
    use crate::Gol;

    // cpu
    #[test]
    fn cpu_structure_tub() {
        structure_tub_tester::<cpu::GameState>();
    }

    #[test]
    fn cpu_structure_box() {
        structure_box_tester::<cpu::GameState>();
    }

    #[test]
    fn cpu_structure_box_wrapped() {
        structure_box_wrapped_tester::<cpu::GameState>();
    }

    #[test]
    fn cpu_structure_blinker() {
        structure_blinker_tester::<cpu::GameState>();
    }

    #[test]
    fn cpu_structure_beacon() {
        structure_beacon_tester::<cpu::GameState>();
    }

    // cpu_ndarray
    #[test]
    fn cpu_ndarray_structure_tub() {
        structure_tub_tester::<cpu_ndarray::GameState>();
    }

    #[test]
    fn cpu_ndarray_ndarray_structure_box() {
        structure_box_tester::<cpu_ndarray::GameState>();
    }

    #[test]
    fn cpu_ndarray_structure_box_wrapped() {
        structure_box_wrapped_tester::<cpu_ndarray::GameState>();
    }

    #[test]
    fn cpu_ndarray_structure_blinker() {
        structure_blinker_tester::<cpu_ndarray::GameState>();
    }

    #[test]
    fn cpu_ndarray_structure_beacon() {
        structure_beacon_tester::<cpu_ndarray::GameState>();
    }

    // compute
    #[test]
    fn compute_structure_tub() {
        structure_tub_tester::<compute::GameState>();
    }

    #[test]
    fn compute_structure_box() {
        structure_box_tester::<compute::GameState>();
    }

    #[test]
    fn compute_structure_box_wrapped() {
        structure_box_wrapped_tester::<compute::GameState>();
    }

    #[test]
    fn compute_structure_blinker() {
        structure_blinker_tester::<compute::GameState>();
    }

    #[test]
    fn compute_structure_beacon() {
        structure_beacon_tester::<compute::GameState>();
    }

    fn structure_tub_tester<T: Gol>() {
        let start = vec![
            false, false, false, false, false, //
            false, false, true, false, false, //
            false, true, false, true, false, //
            false, false, true, false, false, //
            false, false, false, false, false,
        ];

        let state1 = T::from_slice(5, &start);
        state1.print();
        let state2 = state1.to_next();
        state2.print();

        assert!(state2.to_vec() == start);
    }

    fn structure_box_tester<T: Gol>() {
        let start = vec![
            false, false, false, false, //
            false, true, true, false, //
            false, true, true, false, //
            false, false, false, false,
        ];

        let state1 = T::from_slice(4, &start);
        state1.print();
        let state2 = state1.to_next();
        state2.print();

        assert!(state2.to_vec() == start);
    }

    // same as box test, but in the game corner to test wrapping behavior.
    fn structure_box_wrapped_tester<T: Gol>() {
        let start = vec![
            true, false, false, true, //
            false, false, false, false, //
            false, false, false, false, //
            true, false, false, true,
        ];

        let state1 = T::from_slice(4, &start);
        state1.print();
        let state2 = state1.to_next();
        state2.print();

        assert!(state2.to_vec() == start);
    }

    fn structure_blinker_tester<T: Gol>() {
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

        let state1 = T::from_slice(5, &start);
        state1.print();
        let state2 = state1.to_next();
        state2.print();
        let state3 = state2.to_next();
        state3.print();

        assert!(state2.to_vec() == expected_mid);
        // verify that it repeats in 2 cycles
        assert!(state3.to_vec() == start);
    }

    fn structure_beacon_tester<T: Gol>() {
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

        let state1 = T::from_slice(6, &start);
        state1.print();
        let state2 = state1.to_next();
        state2.print();
        let state3 = state2.to_next();
        state3.print();

        assert!(state2.to_vec() == expected_mid);
        // verify that it repeats in 2 cycles
        assert!(state3.to_vec() == start);
    }
}
