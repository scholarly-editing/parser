use std::env;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use config_deserializer::ParserConfigDeserializer;
use parser::config::ParserConfig;

use crate::cli::DEFAULT_CONFIG_NAME;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum ResolveError {
    ConfigNotFound(PathBuf),
    ConfigAutoNotFound,
    ConfigLoad { path: PathBuf, source: Box<dyn Error> },
    InputNotFound(PathBuf),
    NoInputFiles,
    Io(std::io::Error),
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigNotFound(p) => write!(f, "config file not found: {}", p.display()),
            Self::ConfigAutoNotFound => write!(
                f,
                "no config file specified and {DEFAULT_CONFIG_NAME} not found in current directory.\n\
                 Use --config <PATH> to specify a config file."
            ),
            Self::ConfigLoad { path, source } => {
                write!(f, "failed to load config '{}': {source}", path.display())
            }
            Self::InputNotFound(p) => write!(f, "input file not found: {}", p.display()),
            Self::NoInputFiles => {
                write!(f, "no .txt input files found. Use --input <FILE> to specify files.")
            }
            Self::Io(e) => write!(f, "{e}"),
        }
    }
}

impl Error for ResolveError {}

impl From<std::io::Error> for ResolveError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

// ---------------------------------------------------------------------------
// Config resolution
// ---------------------------------------------------------------------------

/// Resolve the config file path.
///
/// If `explicit` is `Some`, validates that the file exists.
/// Otherwise, looks for `sep-config.toml` in the current directory.
pub fn resolve_config_path(explicit: Option<&Path>) -> Result<PathBuf, ResolveError> {
    match explicit {
        Some(p) => {
            if !p.exists() {
                return Err(ResolveError::ConfigNotFound(p.to_path_buf()));
            }
            Ok(p.to_path_buf())
        }
        None => {
            let default = env::current_dir()?.join(DEFAULT_CONFIG_NAME);
            if !default.exists() {
                return Err(ResolveError::ConfigAutoNotFound);
            }
            Ok(default)
        }
    }
}

/// Load and parse the TOML config file into a [`ParserConfig`].
pub fn load_config(path: &Path) -> Result<ParserConfig, ResolveError> {
    let path_str = path.to_string_lossy();
    ParserConfigDeserializer::from_toml_file(&path_str)
        .map(ParserConfig::from)
        .map_err(|source| ResolveError::ConfigLoad {
            path: path.to_path_buf(),
            source,
        })
}

// ---------------------------------------------------------------------------
// Input file resolution
// ---------------------------------------------------------------------------

/// Resolve the list of input files.
///
/// If `explicit` is `Some`, validates every path exists.
/// Otherwise, discovers all `*.txt` files in the current directory (sorted).
pub fn resolve_input_files(explicit: Option<&[PathBuf]>) -> Result<Vec<PathBuf>, ResolveError> {
    let files = match explicit {
        Some(paths) => {
            for p in paths {
                if !p.exists() {
                    return Err(ResolveError::InputNotFound(p.clone()));
                }
            }
            paths.to_vec()
        }
        None => discover_txt_files(&env::current_dir()?)?,
    };

    if files.is_empty() {
        return Err(ResolveError::NoInputFiles);
    }
    Ok(files)
}

/// Discover all `.txt` files in `dir`, sorted by name.
fn discover_txt_files(dir: &Path) -> Result<Vec<PathBuf>, ResolveError> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "txt") {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    files.sort();
    Ok(files)
}

// ---------------------------------------------------------------------------
// Output directory resolution
// ---------------------------------------------------------------------------

/// Resolve and create the output directory. Relative paths are resolved
/// against the current working directory.
pub fn resolve_output_dir(dir: &Path) -> Result<PathBuf, ResolveError> {
    let abs = if dir.is_relative() {
        env::current_dir()?.join(dir)
    } else {
        dir.to_path_buf()
    };
    fs::create_dir_all(&abs)?;
    Ok(abs)
}
