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
pub enum Win {
    Left,
    Right,
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

    pub fn update<I>(&mut self, events: I) -> Option<Win>
    where
        I: Iterator<Item = Event>,
    {
        for event in events {
            self.handle_event(event);
        }
        self.ball.handle_edge_bounce(&self.root_rectangle);
        self.ball
            .handle_paddle_bounce(&self.left_paddle, Side::Left, &self.root_rectangle);
        self.ball
            .handle_paddle_bounce(&self.right_paddle, Side::Right, &self.root_rectangle);
        self.ball.update_position();
        self.ball.handle_win(&self.root_rectangle)
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::LeftPaddle(event) => self.left_paddle.handle_event(&self.root_rectangle, event),
            Event::RightPaddle(event) => {
                self.right_paddle.handle_event(&self.root_rectangle, event)
            }
        }
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

    fn handle_event(&mut self, root: &Rectangle, event: PaddleEvent) {
        let y_change = match event {
            PaddleEvent::MoveUp => -0.5,
            PaddleEvent::MoveDown => 0.5,
        };
        let moved = self.0.moved_by(Vector2::new(0., y_change));
        if moved.overlaps(&root.grow(-1.)) {
            self.0 = moved;
        }
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
            direction: Vector2::new(1., 0.).normalize(),
        }
    }

    fn handle_win(&mut self, root: &Rectangle) -> Option<Win> {
        let Self { rectangle, .. } = self;

        if root.overlaps(rectangle) {
            None
        } else {
            Some(if rectangle.center().x < root.center().x {
                // ball on left
                Win::Right
            } else {
                // ball on right
                Win::Left
            })
        }
    }

    fn update_position(&mut self) {
        let Self {
            direction,
            rectangle,
        } = self;
        *rectangle = rectangle.moved_by(*direction);
    }

    fn handle_edge_bounce(&mut self, root: &Rectangle) {
        let Self {
            direction,
            rectangle,
        } = self;

        let moved = rectangle.moved_by(*direction);

        if !root.overlaps(&moved) {
            let normal = Vector2::new(0., moved.center().y.signum()).normalize();

            let reflected_direction = reflect(*direction, normal);

            *direction = reflected_direction.normalize();
        }
    }

    fn handle_paddle_bounce(&mut self, paddle: &Paddle, side: Side, root: &Rectangle) {
        let Self {
            direction,
            rectangle,
        } = self;

        let moved = rectangle.moved_by(*direction);

        // TODO: check if ball overlaps with bouding box of moved area
        if paddle.0.overlaps(&moved) {
            let y_distance_from_center = root.center().y - paddle.0.center().y;
            let height = root.dimensions().y;

            let normal = Vector2::new(
                match side {
                    Side::Left => 1.,
                    Side::Right => -1.,
                },
                y_distance_from_center / height,
            )
            .normalize();

            let reflected_direction = reflect(*direction, normal);

            // increase speed with each bounce
            *direction = reflected_direction * 1.05;
        }
    }
}

impl Render for Ball {
    fn render(&self, terminal: &mut Terminal) {
        self.rectangle.render(terminal)
    }
}

fn reflect(incidence: Vector2<f32>, normal: Vector2<f32>) -> Vector2<f32> {
    // https://math.stackexchange.com/questions/13261/how-to-get-a-reflection-vector
    // reflection = incidence - 2(incidence . normal)normal
    incidence - 2. * incidence.dot(&normal) * normal
}
