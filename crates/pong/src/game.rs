use crate::terminal::{CellKind, Clear, Overlaps, Rectangle, Render, Terminal};
use nalgebra::Vector2;

#[derive(Debug)]
pub enum Event {
    LeftPaddle(PaddleEvent),
    RightPaddle(PaddleEvent),
}

impl Event {
    pub fn from_crossterm(event: crossterm::event::Event) -> Option<Self> {
        use crossterm::event::{Event, KeyCode};

        match event {
            Event::Key(key) => match key.code {
                KeyCode::Up => Some(Self::RightPaddle(PaddleEvent::MoveUp)),
                KeyCode::Down => Some(Self::RightPaddle(PaddleEvent::MoveDown)),
                KeyCode::Char('k') => Some(Self::LeftPaddle(PaddleEvent::MoveUp)),
                KeyCode::Char('j') => Some(Self::LeftPaddle(PaddleEvent::MoveDown)),
                _ => None,
            },
            _ => None,
        }
    }
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

    pub fn update<I>(&mut self, events: I)
    where
        I: Iterator<Item = Event>,
    {
        self.ball.handle_edge_bounce(&self.root_rectangle);
        self.ball.update_position();
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
            direction: Vector2::new(1., 3.).normalize(),
        }
    }

    fn update_position(&mut self) {
        // TODO: make ball bounce from edges, paddles, and detect if someone has lost
        let Self {
            direction,
            rectangle,
        } = self;
        *rectangle = rectangle.moved_by(*direction);
    }

    /// Updates the direction to account for the top and bottom edges
    fn handle_edge_bounce(&mut self, root: &Rectangle) {
        let Self {
            direction,
            rectangle,
        } = self;

        let moved = rectangle.moved_by(*direction);

        if !root.overlaps(&moved) {
            let normal = Vector2::<f32>::new(0., moved.center().y.signum()).normalize();
            let incidence_direction = *direction;

            // https://math.stackexchange.com/questions/13261/how-to-get-a-reflection-vector
            // reflection = incidence - 2(incidence . normal)normal
            let reflected_direction =
                incidence_direction - 2. * incidence_direction.dot(&normal) * normal;

            *direction = reflected_direction.normalize();
        }
    }
}

impl Render for Ball {
    fn render(&self, terminal: &mut Terminal) {
        self.rectangle.render(terminal)
    }
}
