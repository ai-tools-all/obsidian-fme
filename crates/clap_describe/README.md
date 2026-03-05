# clap_describe

A Rust library that adds a `--describe` flag to [clap](https://docs.rs/clap) CLI commands, producing structured schema output (Markdown or JSON) that AI agents and LLMs can consume to understand how to use your tool.

Traditional `--help` output is designed for humans. `--describe` produces machine-readable documentation that agents can parse to discover commands, arguments, options, and subcommands — making your CLI AI-friendly with minimal effort.

Inspired by [Rewrite your CLI for AI Agents](https://justin.poehnelt.com/posts/rewrite-your-cli-for-ai-ag).

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
clap_describe = "0.1"
clap = { version = "4", features = ["derive"] }
```

Flatten `Describe` into your clap command and call `handle()` early in your execution:

```rust
use clap::Parser;

#[derive(Parser)]
#[command(about = "Convert markdown files")]
struct Cli {
    #[command(flatten)]
    describe: clap_describe::Describe,

    /// Input file to process
    #[arg(required = true)]
    input: Option<String>,

    /// Output format
    #[arg(long, default_value = "html", value_parser = ["html", "pdf", "txt"])]
    format: String,

    /// Enable verbose logging
    #[arg(long, short)]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();

    // Check --describe before doing any real work.
    // Second arg: true = JSON output, false = Markdown.
    // Third arg: optional extra description for agents.
    if let Some(output) = cli.describe.handle(
        &Cli::command(),
        false,
        Some("Supports stdin via -".into()),
    ) {
        println!("{output}");
        return;
    }

    // ... normal CLI logic
}
```

## Example: Markdown output

```
$ mycli --describe
```

```markdown
# `mycli`

Convert markdown files

**Usage:** `mycli [OPTIONS] <INPUT>`

## Arguments

| Name | Required | Description |
|------|----------|-------------|
| `INPUT` | yes | Input file to process |

## Options

| Flag | Required | Default | Possible Values | Description |
|------|----------|---------|-----------------|-------------|
| `--format <FORMAT>` | no | html | html, pdf, txt | Output format |
| `-v, --verbose` | no | - | - | Enable verbose logging |

## Notes

Supports stdin via -
```

## Example: JSON output

Pass `true` as the second argument to `handle()` to get JSON:

```json
{
  "name": "mycli",
  "about": "Convert markdown files",
  "usage": "mycli [OPTIONS] <INPUT>",
  "positional_args": [
    {
      "name": "input",
      "required": true,
      "help": "Input file to process",
      "value_name": "INPUT",
      "takes_value": true
    }
  ],
  "options": [
    {
      "name": "format",
      "long": "format",
      "required": false,
      "default_value": "html",
      "possible_values": ["html", "pdf", "txt"],
      "help": "Output format",
      "value_name": "FORMAT",
      "takes_value": true
    },
    {
      "name": "verbose",
      "short": "v",
      "long": "verbose",
      "required": false,
      "help": "Enable verbose logging",
      "takes_value": false
    }
  ],
  "extra_description": "Supports stdin via -"
}
```

## API

| Function | Description |
|----------|-------------|
| `Describe` | Clap `Args` struct that adds `--describe` to your command |
| `Describe::handle(cmd, json, extra)` | Returns `Some(String)` if `--describe` was passed, `None` otherwise |
| `extract_schema(cmd)` | Extract a `CommandSchema` from any `clap::Command` |
| `extract_schema_with_extra(cmd, extra)` | Same, with an optional extra description string |
| `CommandSchema::to_markdown()` | Render the schema as Markdown |

## License

MIT OR Apache-2.0
