use crossterm::{
    cursor::MoveTo,
    style::{Color, PrintStyledContent, Stylize},
    terminal, QueueableCommand,
};
use nalgebra::Vector2;
use ndarray::Array2;
use std::{
    io::{Stdout, Write},
    ops::Range,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct Rectangle {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    cell_kind: CellKind,
}

impl Rectangle {
    fn x_range(&self) -> Range<usize> {
        let start = self.x.floor() as _;
        let end = (self.x + self.width).ceil() as _;
        start..end
    }

    fn y_range(&self) -> Range<usize> {
        let start = self.y.floor() as _;
        let end = (self.y + self.height).ceil() as _;
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

    pub fn move_by(&mut self, x_move: f32, y_move: f32) {
        let Self { x, y, .. } = self;
        *x += x_move;
        *y += y_move;
    }

    pub fn center(&self) -> Vector2<f32> {
        let x = self.x + (self.width / 2.);
        let y = self.y + (self.height / 2.);
        Vector2::new(x, y)
    }
}

impl Render for Rectangle {
    fn render(&self, terminal: &mut Terminal) {
        let x_range = self.x_range();
        let y_range = self.y_range();

        let term_height = terminal.height as usize;
        let term_width = terminal.width as usize;

        for y in 0..term_height {
            if y_range.contains(&y) {
                for x in 0..term_width {
                    if x_range.contains(&x) {
                        *terminal.cells.get_mut([y, x]).unwrap() = self.cell_kind
                    }
                }
            }
        }
    }
}

pub trait Overlaps {
    fn overlaps(&self, other: &Self) -> bool;
}

impl<T> Overlaps for Range<T>
where
    T: std::cmp::Ord + Copy,
{
    fn overlaps(&self, other: &Self) -> bool {
        self.end.min(other.end) >= self.start.max(other.start)
    }
}

#[test]
fn range_overlap() {
    assert!(!(0..10).overlaps(&(11..12)));
    assert!((10..12).overlaps(&(11..30)));
}

impl Overlaps for Rectangle {
    fn overlaps(&self, other: &Self) -> bool {
        let x_overlaps = self.x_range().overlaps(&other.x_range());
        let y_overlaps = self.y_range().overlaps(&other.y_range());
        x_overlaps & y_overlaps
    }
}

#[test]
fn rectange_overlap() {
    let a = Rectangle::new((0.)..3., (0.)..3., CellKind::Empty);
    let b = Rectangle::new((5.)..6., (0.)..3., CellKind::Empty);
    assert!(!a.overlaps(&b));

    let a = Rectangle::new((0.)..3., (0.)..3., CellKind::Empty);
    let b = Rectangle::new(2.9..5., (0.)..3., CellKind::Empty);
    assert!(a.overlaps(&b));

    let a = Rectangle::new((0.)..1., (0.)..100., CellKind::Empty);
    let b = Rectangle::new((0.)..1., (101.)..102., CellKind::Empty);
    assert!(!a.overlaps(&b));
}

#[test]
fn rectange_creation() {
    assert_eq!(
        Rectangle::new((100.)..101., (100.)..101., CellKind::Empty),
        Rectangle {
            x: 100.,
            y: 100.,
            width: 1.,
            height: 1.,
            cell_kind: CellKind::Empty
        }
    )
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
            cells: Array2::from_elem((height as usize, width as usize), CellKind::Empty),
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
