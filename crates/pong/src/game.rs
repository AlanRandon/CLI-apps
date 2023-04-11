use crate::terminal::{CellKind, Rectangle, Render, Terminal};

#[derive(Debug)]
pub enum Event {
    LeftPaddle(PaddleEvent),
    RightPaddle(PaddleEvent),
}

#[derive(Debug)]
pub enum PaddleEvent {
    MoveUp,
    MoveDown,
}

#[derive(Debug)]
pub struct GameState {
    left_paddle: Paddle,
    right_paddle: Paddle,
    ball: Ball,
    width: f32,
    height: f32,
}

impl GameState {
    pub fn new(terminal: &Terminal) -> Self {
        let (width, height) = terminal.dimensions();
        let width = width as f32;
        let height = height as f32;
        Self {
            left_paddle: Paddle::new(Side::Left, height / 2.),
            right_paddle: Paddle::new(Side::Right, height / 2.),
            ball: Ball::new(width / 2., height / 2.),
            width,
            height,
        }
    }
}

impl Render for GameState {
    fn render(&self, terminal: &mut Terminal) {
        terminal.render(&Rectangle::new((0.)..3., (0.)..3., CellKind::Paddle))
    }
}

#[derive(Debug)]
struct Paddle {
    y: f32,
    side: Side,
}

impl Paddle {
    fn new(side: Side, y: f32) -> Self {
        Self { y, side }
    }
}

#[derive(Debug)]
enum Side {
    Left,
    Right,
}

#[derive(Debug)]
struct Ball {
    velocity: (f32, f32),
    position: (f32, f32),
}

impl Ball {
    fn new(x: f32, y: f32) -> Self {
        Self {
            position: (x, y),
            velocity: (0.5, 0.5),
        }
    }
}
