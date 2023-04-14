use crate::terminal::{CellKind, Clear, Overlaps, Rectangle, Render, Terminal};
use crossterm::{cursor::MoveTo, style::Print, ExecutableCommand};
use nalgebra::Vector2;

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
    root_rectangle: Rectangle,
}

impl GameState {
    pub fn new(terminal: &Terminal) -> Self {
        let (width, height) = terminal.dimensions();
        let width = width as f32;
        let height = height as f32;
        Self {
            left_paddle: Paddle::new(Side::Left, height / 2., width),
            right_paddle: Paddle::new(Side::Right, height / 2., width),
            ball: Ball::new(width / 2., height / 2.),
            root_rectangle: Rectangle::new((0.)..width, (0.)..height, CellKind::Empty),
        }
    }

    pub fn update(&mut self) {
        self.ball
            .update(&self.root_rectangle, &self.left_paddle, &self.right_paddle);
    }
}

impl Render for GameState {
    fn render(&self, terminal: &mut Terminal) {
        terminal.render(&Clear);
        terminal.render(&self.left_paddle);
        terminal.render(&self.right_paddle);
        terminal.render(&self.ball);
    }
}

#[derive(Debug)]
struct Paddle(Rectangle);

impl Paddle {
    fn new(side: Side, y: f32, width: f32) -> Self {
        Self(Rectangle::new(
            match side {
                Side::Left => (0.)..1.,
                Side::Right => (width - 1.)..width,
            },
            (y - 3.)..(y + 3.),
            CellKind::Paddle,
        ))
    }
}

impl Render for Paddle {
    fn render(&self, terminal: &mut Terminal) {
        self.0.render(terminal)
    }
}

#[derive(Debug)]
enum Side {
    Left,
    Right,
}

#[derive(Debug)]
struct Ball {
    direction: Vector2<f32>,
    rectangle: Rectangle,
}

impl Ball {
    fn new(x: f32, y: f32) -> Self {
        Self {
            rectangle: Rectangle::new((x - 0.5)..(x + 0.5), (y - 0.5)..(y + 0.5), CellKind::Ball),
            direction: Vector2::new(0.5, 0.5),
        }
    }

    fn update(&mut self, root: &Rectangle, left_paddle: &Paddle, right_paddle: &Paddle) {
        // TODO: make ball bounce from edges, paddles, and detect if someone has lost

        let rectangle = &mut self.rectangle;
        rectangle.move_by(self.direction.x, self.direction.y);

        if !rectangle.overlaps(root) {
            let ball_center = rectangle.center();
            let root_center = root.center();
            self.direction = (root_center - ball_center).normalize();
        }

        // debug current state
        std::io::stdout()
            .execute(MoveTo(0, 0))
            .unwrap()
            .execute(Print(
                format!("{:#?}\n{rectangle:#?}\n{root:#?}", rectangle.overlaps(root)).as_str(),
            ))
            .unwrap();
    }
}

impl Render for Ball {
    fn render(&self, terminal: &mut Terminal) {
        self.rectangle.render(terminal)
    }
}
