mod enforce;
mod frontmatter;
mod model;
mod query;
mod render;
mod sr;

use clap::{CommandFactory, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "md-fme",
    version = env!("CARGO_PKG_VERSION"),
    about = concat!("Frontmatter Engine (md-fme) v", env!("CARGO_PKG_VERSION"), " — a CLI for Obsidian vaults.\n\nEnforce frontmatter schemas, query fields with a rich DSL,\nand run SM-2 spaced repetition workflows — all from the terminal."),
    after_help = r#"QUICK START:
  md-fme enforce --folder ./mistakes/
  md-fme query "difficulty = hard AND status = completed" --folder .
  md-fme today --folder .
  md-fme review --file mistakes/133_clone_graph.md --quality 4
  md-fme stats --folder .
  md-fme stats --folder . --json"#
)]
struct Cli {
    /// Output as JSON (applies to all commands)
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate frontmatter against a TOML schema
    #[command(
        long_about = r#"Validate .md files against a TOML schema.

Scans a folder (recursive up to --depth levels, default 3) and checks every .md file's YAML
frontmatter against the rules in a TOML schema. Auto-discovers
schema.toml in the target folder when --schema is omitted.

Schema format (schema.toml):
  [fields.status]
  mandatory = true
  allowed_values = ["attempted", "completed", "revisited"]

  [fields.date]
  mandatory = true
  format = "date"

  [fields.difficulty]
  mandatory = false
  allowed_values = ["easy", "medium", "hard"]"#,
        after_help = r#"EXAMPLES:
  # Validate using auto-discovered schema.toml
  md-fme enforce --folder ./mistakes/

  # Validate with an explicit schema file
  md-fme enforce --schema ./custom-schema.toml --folder ./mistakes/"#
    )]
    Enforce {
        #[command(flatten)]
        describe: clap_describe::Describe,
        /// Schema file (defaults to <folder>/schema.toml or <file's parent>/schema.toml)
        #[arg(long)]
        schema: Option<PathBuf>,
        /// Single file to validate (mutually exclusive with --folder)
        #[arg(long, conflicts_with = "folder")]
        file: Option<PathBuf>,
        /// Folder to scan (mutually exclusive with --file)
        #[arg(long, conflicts_with = "file")]
        folder: Option<PathBuf>,
        /// Auto-fix missing mandatory fields using schema defaults
        #[arg(long)]
        fix: bool,
        /// Comma-separated file patterns to exclude (e.g. "README.md,template.md")
        #[arg(long)]
        exclude: Option<String>,
        /// Recursion depth (0=unlimited, 1=no recursion, default=3)
        #[arg(long, default_value_t = 3)]
        depth: usize,
    },

    /// Query frontmatter with a DSL expression
    #[command(
        long_about = r#"Query frontmatter with a rich DSL expression.

Scans a folder (recursive up to --depth levels, default 3) and evaluates a boolean expression
against every .md file's YAML frontmatter. Prints matching
file paths (or field values with --verbose).

OPERATORS:
  =          Exact match           "difficulty = hard"
  !=         Not equal             "status != attempted"
  contains   List/string contains  "topics contains bfs"
  <          Less than             "sr.interval < 5"
  <=         Less than or equal    "sr.next_review <= today"
  >          Greater than          "sr.ease > 2.5"
  >=         Greater than or equal "sr.reps >= 3"
  exists     Field is present      "pattern exists"
  missing    Field is absent       "pattern missing"

COMBINATORS:
  AND        Both conditions must match (binds tighter than OR)
  OR         Either condition must match

SPECIAL:
  today      Expands to current date (YYYY-MM-DD)
  dot.path   Access nested YAML fields: sr.next_review, sr.ease"#,
        after_help = r#"EXAMPLES:
  # Find all BFS problems
  md-fme query "topics contains bfs" --folder .

  # Find completed hard problems
  md-fme query "difficulty = hard AND status = completed" --folder .

  # Find items due for review today or earlier
  md-fme query "sr.next_review <= today" --folder .

  # Find files missing a pattern field
  md-fme query "pattern missing" --folder .

  # Combine review_type filter with due date
  md-fme query "review_type = solve AND sr.next_review <= today" --folder .

  # Show field values for matching files
  md-fme query "status = completed" --folder . --verbose"#
    )]
    Query {
        #[command(flatten)]
        describe: clap_describe::Describe,
        /// Query expression (e.g. "difficulty = hard AND status = completed")
        expression: String,
        /// Folder to scan
        #[arg(long)]
        folder: PathBuf,
        /// Show matching field values and skipped-file warnings
        #[arg(long)]
        verbose: bool,
        /// Recursion depth (0=unlimited, 1=no recursion, default=3)
        #[arg(long, default_value_t = 3)]
        depth: usize,
    },

    /// Show SR items due today
    #[command(
        long_about = r#"Show spaced repetition items due for review today.

Sugar for: md-fme query "sr.next_review <= today" --folder <folder>

Outputs a formatted table with file name, next review date,
interval, ease factor, and review type."#,
        after_help = r#"EXAMPLES:
  md-fme today --folder .
  md-fme today --folder ./mistakes/"#
    )]
    Today {
        #[command(flatten)]
        describe: clap_describe::Describe,
        /// Folder to scan
        #[arg(long)]
        folder: PathBuf,
        /// Recursion depth (0=unlimited, 1=no recursion, default=3)
        #[arg(long, default_value_t = 3)]
        depth: usize,
    },

    /// Record a review with SM-2 quality rating
    #[command(
        long_about = r#"Record a review using the SM-2 algorithm.

Updates the file's sr: frontmatter block with recalculated
interval, ease factor, reps count, and next review date
based on the quality rating you provide.

SM-2 QUALITY SCALE:
  0   Complete blackout, no recall at all
  1   Incorrect, but remembered upon seeing the answer
  2   Incorrect, but answer felt familiar
  3   Correct, but required significant effort
  4   Correct, with some hesitation
  5   Perfect recall, no hesitation

Quality 0-2 resets the interval to 1 day (failure).
Quality 3 produces slower interval growth.
Quality 4-5 produces normal/fast interval growth."#,
        after_help = r#"EXAMPLES:
  # Good recall with some hesitation
  md-fme review --file mistakes/133_clone_graph.md --quality 4

  # Perfect recall
  md-fme review --file mistakes/542_multisource_bfs.md --quality 5

  # Failed recall — resets interval
  md-fme review --file mistakes/133_clone_graph.md --quality 1"#
    )]
    Review {
        #[command(flatten)]
        describe: clap_describe::Describe,
        /// File to review
        #[arg(long)]
        file: PathBuf,
        /// Quality rating 0-5
        #[arg(long)]
        quality: u8,
    },

    /// Initialize SR fields on a file or folder
    #[command(
        long_about = r#"Initialize spaced repetition fields on markdown files.

Adds an sr: block to the YAML frontmatter with default values:
  sr:
    next_review: <today>
    interval: 1
    ease: 2.5
    reps: 0
    last_reviewed: <today>
    review_type: <type>

In folder mode, only files that lack an existing sr: block
are modified. Provide --file for a single file or --folder
for batch initialization."#,
        after_help = r#"EXAMPLES:
  # Initialize a single file
  md-fme init-sr --file mistakes/542.md

  # Batch-initialize all files in a folder (default type: recall)
  md-fme init-sr --folder ./mistakes/

  # Batch-initialize with a custom review type
  md-fme init-sr --folder ./mistakes/ --review-type solve"#
    )]
    InitSr {
        #[command(flatten)]
        describe: clap_describe::Describe,
        /// Single file
        #[arg(long)]
        file: Option<PathBuf>,
        /// Batch: all files without sr: block
        #[arg(long)]
        folder: Option<PathBuf>,
        /// Review type (default: recall)
        #[arg(long, default_value = "recall")]
        review_type: String,
        /// Recursion depth (0=unlimited, 1=no recursion, default=3)
        #[arg(long, default_value_t = 3)]
        depth: usize,
    },

    /// Show spaced repetition statistics
    #[command(
        long_about = r#"Show spaced repetition statistics for a folder.

Scans all .md files with sr: frontmatter and displays:
  - Total SR items and how many are due today
  - Overdue items (past their next_review date)
  - Reviews completed this week
  - Current review streak
  - Weakest items (lowest ease factor)
  - 7-day review load forecast"#,
        after_help = r#"EXAMPLES:
  md-fme stats --folder .
  md-fme stats --folder ./mistakes/"#
    )]
    Stats {
        #[command(flatten)]
        describe: clap_describe::Describe,
        /// Folder to scan
        #[arg(long)]
        folder: PathBuf,
        /// Recursion depth (0=unlimited, 1=no recursion, default=3)
        #[arg(long, default_value_t = 3)]
        depth: usize,
    },
}

fn main() {
    let cli = Cli::parse();
    let json = cli.json;

    let describe_output = match &cli.command {
        Commands::Enforce { describe, .. } => {
            describe.handle(Cli::command().find_subcommand("enforce").unwrap(), json, None)
        }
        Commands::Query { describe, .. } => {
            describe.handle(Cli::command().find_subcommand("query").unwrap(), json, None)
        }
        Commands::Today { describe, .. } => {
            describe.handle(Cli::command().find_subcommand("today").unwrap(), json, None)
        }
        Commands::Review { describe, .. } => {
            describe.handle(Cli::command().find_subcommand("review").unwrap(), json, None)
        }
        Commands::InitSr { describe, .. } => {
            describe.handle(Cli::command().find_subcommand("init-sr").unwrap(), json, None)
        }
        Commands::Stats { describe, .. } => {
            describe.handle(Cli::command().find_subcommand("stats").unwrap(), json, None)
        }
    };

    if let Some(out) = describe_output {
        println!("{out}");
        return;
    }

    let result = match cli.command {
        Commands::Enforce { describe: _, schema, file, folder, fix, exclude, depth } => {
            match (file, folder) {
                (Some(f), None) => {
                    let schema_path = schema.unwrap_or_else(|| {
                        f.parent().unwrap_or(std::path::Path::new(".")).join("schema.toml")
                    });
                    enforce::run_single_file(&schema_path, &f, fix, json)
                }
                (None, Some(dir)) => {
                    let schema_path = schema.unwrap_or_else(|| dir.join("schema.toml"));
                    enforce::run(&schema_path, &dir, fix, exclude.as_deref(), depth, json)
                }
                _ => Err("Must specify either --file or --folder".to_string()),
            }
        }
        Commands::Query { describe: _, expression, folder, verbose, depth } => {
            query::run(&expression, &folder, verbose, depth, json)
        }
        Commands::Today { describe: _, folder, depth } => sr::today(&folder, depth, json),
        Commands::Review { describe: _, file, quality } => sr::review(&file, quality, json),
        Commands::InitSr { describe: _, file, folder, review_type, depth } => {
            sr::init_sr(file.as_deref(), folder.as_deref(), &review_type, depth, json)
        }
        Commands::Stats { describe: _, folder, depth } => sr::stats(&folder, depth, json),
    };
    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
