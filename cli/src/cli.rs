use std::path::PathBuf;

use clap::Parser as ClapParser;
use parser::core::state::Mode;

pub const DEFAULT_CONFIG_NAME: &str = "sep-config.toml";
pub const DEFAULT_OUTPUT_DIR: &str = "sep-output";

/// Scholarly Editing Parser — tokenize manuscript transcriptions with editorial markup.
#[derive(ClapParser, Debug)]
#[command(name = "sep", version, about)]
pub struct Cli {
    /// Path to a TOML config file. Defaults to ./sep-config.toml in the current directory.
    #[arg(short, long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// One or more input .txt files. If omitted, all .txt files in the current directory are processed.
    #[arg(short, long, value_name = "FILE", num_args = 1..)]
    pub input: Option<Vec<PathBuf>>,

    /// Output directory for result and error JSON files.
    #[arg(short, long, value_name = "DIR", default_value = DEFAULT_OUTPUT_DIR)]
    pub output_dir: PathBuf,

    /// Parsing mode: single-page, multiple-pages, or passage.
    #[arg(short, long, value_name = "MODE", default_value = "multiple-pages")]
    pub mode: ModeArg,
}

#[derive(Debug, Clone)]
pub enum ModeArg {
    SinglePage,
    MultiplePages,
    Passage,
}

impl From<&ModeArg> for Mode {
    fn from(arg: &ModeArg) -> Self {
        match arg {
            ModeArg::SinglePage => Mode::SinglePage,
            ModeArg::MultiplePages => Mode::MultiplePages,
            ModeArg::Passage => Mode::Passage,
        }
    }
}

impl std::fmt::Display for ModeArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModeArg::SinglePage => write!(f, "single-page"),
            ModeArg::MultiplePages => write!(f, "multiple-pages"),
            ModeArg::Passage => write!(f, "passage"),
        }
    }
}

impl std::str::FromStr for ModeArg {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "single-page" | "singlepage" | "single" => Ok(ModeArg::SinglePage),
            "multiple-pages" | "multiplepages" | "multiple" | "multi" => {
                Ok(ModeArg::MultiplePages)
            }
            "passage" => Ok(ModeArg::Passage),
            _ => Err(format!(
                "invalid mode '{s}': expected single-page, multiple-pages, or passage"
            )),
        }
    }
}
