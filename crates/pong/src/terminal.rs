use crossterm::{
    cursor::MoveTo,
    style::{Color, Print, PrintStyledContent, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
};
use nalgebra::Vector2;
use ndarray::{s, Array2};
use std::{
    io::{Stdout, Write},
    ops::{Add, AddAssign, Range, Sub},
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
pub struct Rectangle<T = f32>
where
    T: std::fmt::Debug + PartialEq + Copy + 'static,
{
    top_left: Vector2<T>,
    bottom_right: Vector2<T>,
    cell_kind: CellKind,
}

impl<T> Rectangle<T>
where
    T: std::fmt::Debug + PartialEq + Copy,
{
    fn ranges(&self) -> Vector2<Range<T>> {
        self.top_left
            .zip_map(&self.bottom_right, |start, end| start..end)
    }

    pub fn new(x: Range<T>, y: Range<T>, cell_kind: CellKind) -> Self {
        Self {
            top_left: Vector2::new(x.start, y.start),
            bottom_right: Vector2::new(x.end, y.end),
            cell_kind,
        }
    }

    pub fn moved_by(&self, amount: Vector2<T>) -> Self
    where
        Vector2<T>: Add<Output = Vector2<T>>,
    {
        Self {
            top_left: self.top_left + amount,
            bottom_right: self.bottom_right + amount,
            cell_kind: self.cell_kind,
        }
    }

    pub fn dimensions(&self) -> Vector2<T>
    where
        T: Sub<Output = T>,
    {
        self.ranges().map(|Range { start, end }| end - start)
    }
}

impl Rectangle<f32> {
    pub fn center(&self) -> Vector2<f32> {
        self.ranges().map(|Range { start, end }| (start + end) / 2.)
    }
}

impl Render for Rectangle<f32> {
    fn render(&self, terminal: &mut Terminal) {
        let terminal_ranges = terminal.rectangle.ranges();
        let ranges = self.ranges();
        let ranges = terminal_ranges.zip_map(&ranges, |terminal_range, range| {
            (range.start as usize).clamp(terminal_range.start, terminal_range.end)
                ..(range.end as usize).clamp(terminal_range.start, terminal_range.end)
        });

        terminal
            .cells
            .slice_mut(s![ranges.y.clone(), ranges.x.clone()])
            .fill(self.cell_kind);
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
    rectangle: Rectangle<usize>,
}

impl Terminal {
    pub fn new() -> std::io::Result<Self> {
        let (width, height) = terminal::size()?;
        Ok(Self::from_size(width, height))
    }

    fn from_size(width: u16, height: u16) -> Self {
        let height = height as usize;
        let width = width as usize;
        Self {
            cells: Array2::from_elem((height, width), CellKind::Empty),
            rectangle: Rectangle::new(0..width, 0..height, CellKind::Empty),
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

    pub fn dimensions(&self) -> Vector2<usize> {
        self.rectangle.dimensions()
    }
}
