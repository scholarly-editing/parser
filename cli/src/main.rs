mod cli;
mod output;
mod resolve;
mod runner;

use std::process;

use clap::Parser as ClapParser;
use parser::core::state::Mode;

use cli::Cli;
use resolve::{load_config, resolve_config_path, resolve_input_files, resolve_output_dir};
use runner::{run, RunContext};

fn main() {
    let args = Cli::parse();

    // --- Resolve all inputs ---
    let config_path = resolve_config_path(args.config.as_deref()).unwrap_or_else(|e| {
        eprintln!("Error: {e}");
        process::exit(1);
    });

    let parser_config = load_config(&config_path).unwrap_or_else(|e| {
        eprintln!("Error: {e}");
        process::exit(1);
    });

    let mode: Mode = (&args.mode).into();

    let input_files =
        resolve_input_files(args.input.as_deref()).unwrap_or_else(|e| {
            eprintln!("Error: {e}");
            process::exit(1);
        });

    let output_dir = resolve_output_dir(&args.output_dir).unwrap_or_else(|e| {
        eprintln!("Error: {e}");
        process::exit(1);
    });

    // --- Print run summary ---
    println!(
        "sep: config={}, mode={}, files={}, output={}",
        config_path.display(),
        args.mode,
        input_files.len(),
        output_dir.display(),
    );
    println!();

    // --- Run ---
    let ctx = RunContext {
        config_path,
        parser_config,
        mode,
        input_files,
        output_dir,
    };

    let summary = run(&ctx);

    // --- Report per-file results ---
    for result in &summary.processed {
        let status = if result.error_count > 0 { "!" } else { "+" };
        println!(
            "  {} {} -> {} tokens, {} errors",
            status,
            result.path.display(),
            result.token_count,
            result.error_count,
        );
    }

    for (path, msg) in &summary.failures {
        eprintln!("  ERROR {}: {msg}", path.display());
    }

    // --- Final summary ---
    println!();
    println!(
        "Done: {} files processed, {} total tokens, {} total errors",
        summary.processed.len(),
        summary.total_tokens(),
        summary.total_errors(),
    );

    if !summary.failures.is_empty() {
        eprintln!("{} file(s) failed", summary.failures.len());
        process::exit(1);
    }
}
