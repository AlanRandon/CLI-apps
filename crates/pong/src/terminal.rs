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
    top_left: Vector2<f32>,
    bottom_right: Vector2<f32>,
    cell_kind: CellKind,
}

impl Rectangle {
    fn ranges(&self) -> Vector2<Range<f32>> {
        self.top_left
            .zip_map(&self.bottom_right, |start, end| start..end)
    }

    pub fn new(x: Range<f32>, y: Range<f32>, cell_kind: CellKind) -> Self {
        Self {
            top_left: Vector2::new(x.start, y.start),
            bottom_right: Vector2::new(x.end, y.end),
            cell_kind,
        }
    }

    pub fn move_by(&mut self, change: Vector2<f32>) {
        let Self {
            top_left,
            bottom_right,
            ..
        } = self;
        *top_left += change;
        *bottom_right += change;
    }

    pub fn center(&self) -> Vector2<f32> {
        self.ranges().map(|Range { start, end }| (start + end) / 2.)
    }

    pub fn dimensions(&self) -> Vector2<f32> {
        self.ranges().map(|Range { start, end }| end - start)
    }
}

impl Render for Rectangle {
    fn render(&self, terminal: &mut Terminal) {
        let ranges = self
            .ranges()
            .map(|Range { start, end }| (start.floor() as usize)..(end.ceil() as usize));

        let term_height = terminal.height as usize;
        let term_width = terminal.width as usize;

        for (x, y) in std::iter::zip(0..term_width, 0..term_height)
            .filter(|(x, y)| ranges.x.contains(x) & ranges.y.contains(y))
        {
            *terminal.cells.get_mut([y, x]).unwrap() = self.cell_kind
        }
    }
}

pub trait Overlaps {
    fn overlaps(&self, other: &Self) -> bool;
}

impl Overlaps for Range<usize> {
    fn overlaps(&self, other: &Self) -> bool {
        self.end.min(other.end) >= self.start.max(other.start)
    }
}

impl Overlaps for Range<f32> {
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
        self.ranges()
            .zip_map(&other.ranges(), |a, b| a.overlaps(&b))
            .fold(true, std::ops::BitAnd::bitand)
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
            top_left: Vector2::new(100., 100.),
            bottom_right: Vector2::new(101., 101.),
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
