use std::path::PathBuf;

mod batch;
mod output;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use skin_core::analyze_path;
use skin_rules::{AspectProtection, ConversionOptions, TargetAspect, dry_run, generate};

use batch::run_batch;
use output::{print_analysis, print_conversion_report};

#[derive(Debug, Parser)]
#[command(name = "skin-cli")]
#[command(about = "Offline LR2-to-beatoraja skin conversion workbench")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Analyze {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        json: bool,
    },
    DryRun {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        preset: Option<String>,
        #[arg(long = "rule")]
        rules: Vec<String>,
        #[arg(long)]
        lift_cover_strategy: Option<String>,
        #[arg(long)]
        resolution: Option<u8>,
        #[arg(long)]
        target_aspect: Option<TargetAspect>,
        #[arg(long)]
        preserve_bomb_aspect: bool,
        #[arg(long)]
        preserve_nowjudge_aspect: bool,
        #[arg(long)]
        preserve_nowcombo_aspect: bool,
        #[arg(long)]
        json: bool,
    },
    Generate {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        preset: Option<String>,
        #[arg(long = "rule")]
        rules: Vec<String>,
        #[arg(long)]
        lift_cover_strategy: Option<String>,
        #[arg(long)]
        resolution: Option<u8>,
        #[arg(long)]
        target_aspect: Option<TargetAspect>,
        #[arg(long)]
        preserve_bomb_aspect: bool,
        #[arg(long)]
        preserve_nowjudge_aspect: bool,
        #[arg(long)]
        preserve_nowcombo_aspect: bool,
        #[arg(long)]
        json: bool,
    },
    Batch {
        #[arg(long)]
        job: PathBuf,
        #[arg(long)]
        fail_fast: bool,
        #[arg(long)]
        json: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Analyze { input, json } => {
            let report =
                analyze_path(&input).with_context(|| format!("analyze {}", input.display()))?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_analysis(&report);
            }
        }
        Commands::DryRun {
            input,
            preset,
            rules,
            lift_cover_strategy,
            resolution,
            target_aspect,
            preserve_bomb_aspect,
            preserve_nowjudge_aspect,
            preserve_nowcombo_aspect,
            json,
        } => {
            let options = ConversionOptions {
                preset,
                rules,
                lift_cover_strategy,
                resolution,
                target_aspect,
                aspect_protection: AspectProtection {
                    bomb_effects: preserve_bomb_aspect,
                    nowjudge: preserve_nowjudge_aspect,
                    nowcombo: preserve_nowcombo_aspect,
                },
            };
            let report =
                dry_run(&input, options).with_context(|| format!("dry-run {}", input.display()))?;
            print_conversion_report(&report, json)?;
        }
        Commands::Generate {
            input,
            output,
            preset,
            rules,
            lift_cover_strategy,
            resolution,
            target_aspect,
            preserve_bomb_aspect,
            preserve_nowjudge_aspect,
            preserve_nowcombo_aspect,
            json,
        } => {
            let options = ConversionOptions {
                preset,
                rules,
                lift_cover_strategy,
                resolution,
                target_aspect,
                aspect_protection: AspectProtection {
                    bomb_effects: preserve_bomb_aspect,
                    nowjudge: preserve_nowjudge_aspect,
                    nowcombo: preserve_nowcombo_aspect,
                },
            };
            let report = generate(&input, &output, options)
                .with_context(|| format!("generate {} -> {}", input.display(), output.display()))?;
            print_conversion_report(&report, json)?;
        }
        Commands::Batch {
            job,
            fail_fast,
            json,
        } => {
            let summary = run_batch(&job, fail_fast)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&summary)?);
            } else {
                for entry in summary.entries {
                    if entry.ok {
                        println!(
                            "OK {} -> {} (changed files: {}, warnings: {})",
                            entry.input.display(),
                            entry.output.display(),
                            entry.changed_files,
                            entry.warnings
                        );
                    } else {
                        println!(
                            "FAIL {} -> {}: {}",
                            entry.input.display(),
                            entry.output.display(),
                            entry.error.unwrap_or_else(|| "unknown error".to_owned())
                        );
                    }
                }
            }
        }
    }
    Ok(())
}
