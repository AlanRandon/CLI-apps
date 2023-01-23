mod simple_example;
mod triangle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Design {
    SimpleExample,
    Triangle,
}

impl Design {
    pub fn draw(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::SimpleExample => simple_example::draw(),
            Self::Triangle => triangle::draw(),
        }
    }
}
