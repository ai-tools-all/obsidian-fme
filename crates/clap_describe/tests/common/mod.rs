use clap::{Arg, ArgAction, Command};

/// Minimal command — no args, no subcommands
pub fn empty_command() -> Command {
    Command::new("empty")
}

/// Command with only positional args
pub fn positional_only_command() -> Command {
    Command::new("positional-only")
        .about("Command with positional args only")
        .arg(Arg::new("input").required(true).help("Input file path"))
        .arg(Arg::new("output").required(false).help("Output file path"))
}

/// Realistic CLI resembling md-fme enforce
pub fn realistic_command() -> Command {
    Command::new("enforce")
        .about("Validate frontmatter against a schema")
        .long_about("Validate that markdown files in a folder have frontmatter matching the specified schema. Reports errors and optionally fixes them.")
        .arg(
            Arg::new("schema")
                .long("schema")
                .short('s')
                .value_name("PATH")
                .help("Path to schema file"),
        )
        .arg(
            Arg::new("folder")
                .long("folder")
                .short('f')
                .value_name("FOLDER")
                .default_value(".")
                .help("Folder to scan"),
        )
        .arg(
            Arg::new("fix")
                .long("fix")
                .action(ArgAction::SetTrue)
                .help("Auto-fix issues where possible"),
        )
        .arg(
            Arg::new("exclude")
                .long("exclude")
                .short('e')
                .value_name("PATTERN")
                .help("Glob pattern to exclude"),
        )
        .arg(
            Arg::new("depth")
                .long("depth")
                .short('d')
                .value_name("N")
                .default_value("10")
                .help("Maximum directory depth"),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .value_parser(["json", "text", "github"])
                .default_value("text")
                .help("Output format"),
        )
        .arg(
            Arg::new("hidden-internal")
                .long("hidden-internal")
                .hide(true)
                .help("Internal flag not shown"),
        )
}

/// Command with subcommands (like a top-level CLI)
pub fn command_with_subcommands() -> Command {
    Command::new("mycli")
        .about("A multi-tool CLI")
        .subcommand(Command::new("init").about("Initialize project"))
        .subcommand(Command::new("build").about("Build the project"))
        .subcommand(
            Command::new("deploy")
                .about("Deploy to target")
                .arg(
                    Arg::new("target")
                        .long("target")
                        .value_parser(["staging", "production"])
                        .required(true)
                        .help("Deployment target"),
                ),
        )
        .subcommand(Command::new("hidden-cmd").about("Secret").hide(true))
}

/// Deeply nested subcommands
pub fn nested_subcommands() -> Command {
    Command::new("deep")
        .about("Deeply nested CLI")
        .subcommand(
            Command::new("level1")
                .about("First level")
                .subcommand(
                    Command::new("level2")
                        .about("Second level")
                        .arg(Arg::new("flag").long("flag").action(ArgAction::SetTrue)),
                ),
        )
}
