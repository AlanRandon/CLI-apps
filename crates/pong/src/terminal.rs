use crossterm::{
    cursor::MoveTo,
    style::{Color, PrintStyledContent, Stylize},
    terminal, QueueableCommand,
};
use ndarray::Array2;
use std::{
    io::{Stdout, Write},
    ops::Range,
};

#[derive(Clone, Copy)]
pub enum CellKind {
    Ball,
    Paddle,
    Empty,
}

impl CellKind {
    fn queue_command(&self, stdout: &mut Stdout) -> crossterm::Result<()> {
        stdout.queue(PrintStyledContent(" ".on(match self {
            Self::Ball => Color::Cyan,
            Self::Paddle => Color::White,
            Self::Empty => Color::Reset,
        })))?;
        Ok(())
    }
}

pub struct Rectangle {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    cell_kind: CellKind,
}

impl Rectangle {
    fn x_range(&self) -> Range<usize> {
        let start = self.x.max(0.) as _;
        let end = (self.x + self.width).max(0.) as _;
        start..end
    }

    fn y_range(&self) -> Range<usize> {
        let start = self.y.max(0.) as _;
        let end = (self.y + self.height).max(0.) as _;
        start..end
    }

    pub fn new(x: Range<f32>, y: Range<f32>, cell_kind: CellKind) -> Self {
        Self {
            x: x.start,
            y: y.start,
            width: x.end - x.start,
            height: y.end - y.start,
            cell_kind,
        }
    }
}

impl Render for Rectangle {
    fn render(&self, terminal: &mut Terminal) {
        let x_range = self.x_range();
        for y in self.y_range() {
            for x in x_range.clone() {
                *terminal.cells.get_mut([x, y]).unwrap() = self.cell_kind
            }
        }
    }
}

pub struct Clear;

impl Render for Clear {
    fn render(&self, terminal: &mut Terminal) {
        terminal.cells.fill(CellKind::Empty)
    }
}

pub trait Render {
    fn render(&self, terminal: &mut Terminal);
}

pub struct Terminal {
    cells: Array2<CellKind>,
    width: u16,
    height: u16,
}

impl Terminal {
    pub fn new() -> std::io::Result<Self> {
        let (width, height) = terminal::size()?;
        Ok(Self::from_size(width, height))
    }

    fn from_size(width: u16, height: u16) -> Self {
        Self {
            cells: Array2::from_elem((width as usize, height as usize), CellKind::Empty),
            width,
            height,
        }
    }

    pub fn render_to_stdout(&self, stdout: &mut Stdout) -> crossterm::Result<()> {
        stdout.queue(MoveTo(0, 0))?;
        for cell in self.cells.iter() {
            cell.queue_command(stdout)?;
        }
        stdout.flush()?;
        Ok(())
    }

    pub fn render<T>(&mut self, object: &T)
    where
        T: Render,
    {
        object.render(self);
    }

    pub fn dimensions(&self) -> (u16, u16) {
        (self.width, self.height)
    }
}
