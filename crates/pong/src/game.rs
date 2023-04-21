use crate::terminal::{CellKind, Clear, Overlaps, Rectangle, Render, Terminal};
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
        let terminal_dimensions = terminal.dimensions();
        let width = terminal_dimensions.x as f32;
        let height = terminal_dimensions.y as f32;
        Self {
            left_paddle: Paddle::new(Side::Left, height / 2., width),
            right_paddle: Paddle::new(Side::Right, height / 2., width),
            ball: Ball::new(width / 2., height / 2.),
            root_rectangle: Rectangle::new((0.)..width, (0.)..height, CellKind::Empty),
        }
    }

    /// May return a debugging string
    pub fn update(&mut self) -> Option<String> {
        self.ball
            .update(&self.root_rectangle, &self.left_paddle, &self.right_paddle);

        Some(format!("{:?}", self.ball.direction))
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
            direction: Vector2::new(1., 1.).normalize(),
        }
    }

    fn update(&mut self, root: &Rectangle, left_paddle: &Paddle, right_paddle: &Paddle) {
        // TODO: make ball bounce from edges, paddles, and detect if someone has lost

        // FIXME: do not bounce if going toward center

        let rectangle = &mut self.rectangle;
        let direction = &mut self.direction;

        let ball_center = rectangle.center();
        let root_center = root.center();
        let y_offset = root_center.y - ball_center.y;

        if !root.overlaps(rectangle) & direction.y {
            let incidence_direction = *direction;

            let normal = if y_offset.is_sign_positive() {
                // ball below top
                Vector2::new(1., 0.)
            } else {
                // ball above top
                Vector2::new(-1., 0.)
            }
            .normalize();

            // https://math.stackexchange.com/questions/13261/how-to-get-a-reflection-vector
            // reflection = incidence - 2(incidence . normal)normal
            let reflected_direction =
                incidence_direction - 2. * incidence_direction.dot(&normal) * normal;

            *direction = reflected_direction.normalize();
        }

        *rectangle = rectangle.moved_by(self.direction * 2.);
    }
}

impl Render for Ball {
    fn render(&self, terminal: &mut Terminal) {
        self.rectangle.render(terminal)
    }
}
