mod simple_example;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Design {
    SimpleExample,
}

impl Design {
    pub fn draw(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Self::SimpleExample => simple_example::draw(),
        }
    }
}
