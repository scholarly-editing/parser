use std::fs;
use std::path::{Path, PathBuf};

use parser::config::ParserConfig;
use parser::core::orchestrator::Orchestrator;
use parser::core::state::Mode;

use crate::output::{extract_errors, write_json, ResultOutput};

// ---------------------------------------------------------------------------
// Run context
// ---------------------------------------------------------------------------

/// Resolved parameters for a single `sep` invocation.
#[allow(dead_code)]
pub struct RunContext {
    pub config_path: PathBuf,
    pub parser_config: ParserConfig,
    pub mode: Mode,
    pub input_files: Vec<PathBuf>,
    pub output_dir: PathBuf,
}

// ---------------------------------------------------------------------------
// Per-file result
// ---------------------------------------------------------------------------

/// Outcome of processing a single input file.
pub struct FileResult {
    pub path: PathBuf,
    pub token_count: usize,
    pub error_count: usize,
}

/// Aggregate summary returned by [`run`].
pub struct RunSummary {
    pub processed: Vec<FileResult>,
    pub failures: Vec<(PathBuf, String)>,
}

impl RunSummary {
    pub fn total_tokens(&self) -> usize {
        self.processed.iter().map(|r| r.token_count).sum()
    }

    pub fn total_errors(&self) -> usize {
        self.processed.iter().map(|r| r.error_count).sum()
    }
}

// ---------------------------------------------------------------------------
// Core runner
// ---------------------------------------------------------------------------

/// Process every input file: tokenize, serialize, and write output.
pub fn run(ctx: &RunContext) -> RunSummary {
    let mut processed = Vec::new();
    let mut failures: Vec<(PathBuf, String)> = Vec::new();

    for file_path in &ctx.input_files {
        match process_file(file_path, &ctx.parser_config, &ctx.mode, &ctx.output_dir) {
            Ok(result) => processed.push(result),
            Err(msg) => failures.push((file_path.clone(), msg)),
        }
    }

    RunSummary {
        processed,
        failures,
    }
}

/// Process a single input file end-to-end.
fn process_file(
    file_path: &Path,
    config: &ParserConfig,
    mode: &Mode,
    output_dir: &Path,
) -> Result<FileResult, String> {
    let stem = file_path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();

    // Read input
    let input = fs::read_to_string(file_path)
        .map_err(|e| format!("cannot read file: {e}"))?;

    // Tokenize
    let orchestrator = Orchestrator::new(config, mode);
    let state = orchestrator.tokenize_into_state(&input);

    let token_count = state.tokens.len();
    let error_count = state.errors.len();

    // Write result JSON
    let result = ResultOutput::from_state(&state);
    let result_path = output_dir.join(format!("{stem}.result.json"));
    write_json(&result_path, &result)
        .map_err(|e| format!("failed to write {}: {e}", result_path.display()))?;

    // Write errors JSON
    let errors = extract_errors(&state);
    let errors_path = output_dir.join(format!("{stem}.errors.json"));
    write_json(&errors_path, &errors)
        .map_err(|e| format!("failed to write {}: {e}", errors_path.display()))?;

    Ok(FileResult {
        path: file_path.to_path_buf(),
        token_count,
        error_count,
    })
}
