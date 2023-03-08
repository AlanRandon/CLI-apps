use super::*;

#[test]
fn cells_die_underpopulation() {
    let mut state = State {
        cells: vec![
            // row 0
            CellState::Dead,
            CellState::Dead,
            CellState::Dead,
            // row 1
            CellState::Dead,
            CellState::Alive,
            CellState::Dead,
            // row 2
            CellState::Dead,
            CellState::Dead,
            CellState::Dead,
        ],
        width: 3,
        height: 3,
    };

    state.next_state().for_each(drop);

    assert_eq!(
        state.cells,
        vec![
            CellState::Dead,
            CellState::Dead,
            CellState::Dead,
            // row 1
            CellState::Dead,
            CellState::Dead,
            CellState::Dead,
            // row 2
            CellState::Dead,
            CellState::Dead,
            CellState::Dead,
        ]
    );
}

#[test]
fn cells_die_overpopulation() {
    let mut state = State {
        cells: vec![
            // row 0
            CellState::Alive,
            CellState::Alive,
            CellState::Alive,
            // row 1
            CellState::Alive,
            CellState::Alive,
            CellState::Alive,
            // row 2
            CellState::Alive,
            CellState::Alive,
            CellState::Alive,
        ],
        width: 3,
        height: 3,
    };

    state.next_state().for_each(drop);

    assert_eq!(
        state.cells,
        vec![
            CellState::Dead,
            CellState::Dead,
            CellState::Dead,
            // row 1
            CellState::Dead,
            CellState::Dead,
            CellState::Dead,
            // row 2
            CellState::Dead,
            CellState::Dead,
            CellState::Dead,
        ]
    );
}

#[test]
fn cells_stay_alive() {
    let stable_state = vec![
        // row 0
        CellState::Alive,
        CellState::Dead,
        CellState::Alive,
        // row 1
        CellState::Dead,
        CellState::Dead,
        CellState::Dead,
        // row 2
        CellState::Alive,
        CellState::Dead,
        CellState::Alive,
    ];

    let mut state = State {
        cells: stable_state.clone(),
        width: 3,
        height: 3,
    };

    state.next_state().for_each(drop);

    assert_eq!(state.cells, stable_state);
}
