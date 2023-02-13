#[derive(Clone, Copy)]
pub struct Cell {
    pub alive: bool,
    pub will_stay_alive: bool,
}

impl Cell {
    pub fn new(alive: bool) -> Self {
        Self {
            alive,
            will_stay_alive: false,
        }
    }

    pub fn tick(mut self) -> Self {
        self.alive = self.will_stay_alive;
        self
    }
}
