use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Default)]
#[clap(
    author = "Lucas Waddell",
    version,
    about = "A tool to sort images based on metadata or Google Takeout JSON files."
)]
pub struct Arguments {
    /// Path to the directory containing images or JSON files
    #[clap(
        short,
        long,
        help = "Path to the directory containing images or JSON files"
    )]
    pub path: PathBuf,

    /// Sort images by months
    #[clap(short, help = "Sort images by months")]
    pub months: bool,

    /// Sort images by years
    #[clap(short, help = "Sort images by years")]
    pub years: bool,
}

impl Arguments {
    pub fn validate(&self) -> Result<&Self, String> {
        if !self.path.exists() {
            return Err(format!("The path {:?} does not exist.", self.path));
        }
        if !self.path.is_dir() {
            return Err(format!("{:?} is not a directory.", self.path));
        }
        if !self.years && !self.months {
            return Err(String::from("Either the months or years flag must be set"));
        }

        Ok(self)
    }
}
