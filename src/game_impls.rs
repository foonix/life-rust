pub mod cpu;

#[cfg(test)]
mod tests {
    use super::cpu;
    use super::cpu::GameState;
    use crate::Gol;

    fn structure_tub_tester<T: Gol>() {
        let start = vec![
            false, false, false, false, false, //
            false, false, true, false, false, //
            false, true, false, true, false, //
            false, false, true, false, false, //
            false, false, false, false, false,
        ];

        let state1 = T::from_vec(5, &start);
        state1.print();
        let state2 = T::from_previous(&state1);
        state2.print();

        assert!(state2.to_vec() == start);
    }

    #[test]
    fn cpu_structure_tub() {
        structure_tub_tester::<GameState>();
    }

    fn structure_box_tester<T: Gol>() {
        let start = vec![
            false, false, false, false, //
            false, true, true, false, //
            false, true, true, false, //
            false, false, false, false,
        ];

        let state1 = T::from_vec(4, &start);
        state1.print();
        let state2 = T::from_previous(&state1);
        state2.print();

        assert!(state2.to_vec() == start);
    }

    #[test]
    fn cpu_structure_box() {
        structure_box_tester::<cpu::GameState>();
    }

    // same as box test, but in the game corner to test wrapping behavior.
    fn structure_box_wrapped_tester<T: Gol>() {
        let start = vec![
            true, false, false, true, //
            false, false, false, false, //
            false, false, false, false, //
            true, false, false, true,
        ];

        let state1 = T::from_vec(4, &start);
        state1.print();
        let state2 = T::from_previous(&state1);
        state2.print();

        assert!(state2.to_vec() == start);
    }

    #[test]
    fn cpu_structure_box_wrapped() {
        structure_box_wrapped_tester::<GameState>();
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

        let state1 = T::from_vec(5, &start);
        state1.print();
        let state2 = T::from_previous(&state1);
        state2.print();
        let state3 = T::from_previous(&state2);
        state3.print();

        assert!(state2.to_vec() == expected_mid);
        // verify that it repeats in 2 cycles
        assert!(state3.to_vec() == start);
    }

    #[test]
    fn cpu_structure_blinker() {
        structure_blinker_tester::<GameState>();
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

        let state1 = T::from_vec(6, &start);
        state1.print();
        let state2 = T::from_previous(&state1);
        state2.print();
        let state3 = T::from_previous(&state2);
        state3.print();

        assert!(state2.to_vec() == expected_mid);
        // verify that it repeats in 2 cycles
        assert!(state3.to_vec() == start);
    }

    #[test]
    fn cpu_structure_beacon() {
        structure_beacon_tester::<GameState>();
    }
}
