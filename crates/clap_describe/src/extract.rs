use crate::model::*;

pub fn extract_schema(cmd: &clap::Command) -> CommandSchema {
    extract_schema_with_extra(cmd, None)
}

pub fn extract_schema_with_extra(cmd: &clap::Command, extra: Option<String>) -> CommandSchema {
    let positional_args: Vec<ArgSchema> = cmd
        .get_positionals()
        .filter(|a| !a.is_hide_set())
        .map(extract_arg)
        .collect();

    let options: Vec<ArgSchema> = cmd
        .get_arguments()
        .filter(|a| !a.is_positional() && !a.is_hide_set())
        .map(extract_arg)
        .collect();

    let subcommands: Vec<SubcommandSummary> = cmd
        .get_subcommands()
        .filter(|s| !s.is_hide_set())
        .map(|s| SubcommandSummary {
            name: s.get_name().to_owned(),
            about: s.get_about().map(|a| a.to_string()),
        })
        .collect();

    CommandSchema {
        name: cmd.get_name().to_owned(),
        about: cmd.get_about().map(|a| a.to_string()),
        long_about: cmd.get_long_about().map(|a| a.to_string()),
        usage: cmd.clone().render_usage().to_string().replace("Usage: ", ""),
        positional_args,
        options,
        subcommands,
        extra_description: extra,
    }
}

fn extract_arg(arg: &clap::Arg) -> ArgSchema {
    let possible_values: Vec<String> = arg
        .get_possible_values()
        .into_iter()
        .filter(|pv| !pv.is_hide_set())
        .map(|pv| pv.get_name().to_owned())
        .collect();

    let default_value = arg
        .get_default_values()
        .first()
        .map(|v| v.to_string_lossy().to_string());

    let value_name = arg
        .get_value_names()
        .and_then(|names| names.first().map(|n| n.to_string()));

    ArgSchema {
        name: arg.get_id().to_string(),
        short: arg.get_short(),
        long: arg.get_long().map(|s| s.to_owned()),
        required: arg.is_required_set(),
        default_value,
        possible_values,
        help: arg
            .get_long_help()
            .or_else(|| arg.get_help())
            .map(|h| h.to_string()),
        value_name,
        takes_value: arg.get_action().takes_values(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Arg, Command};

    #[test]
    fn extracts_command_metadata() {
        let cmd = Command::new("testcli")
            .about("Short description")
            .long_about("Long detailed description");

        let schema = extract_schema(&cmd);
        assert_eq!(schema.name, "testcli");
        assert_eq!(schema.about.as_deref(), Some("Short description"));
        assert_eq!(schema.long_about.as_deref(), Some("Long detailed description"));
    }

    #[test]
    fn extracts_positional_args() {
        let cmd = Command::new("testcli").arg(
            Arg::new("input")
                .required(true)
                .help("The input file"),
        );

        let schema = extract_schema(&cmd);
        assert_eq!(schema.positional_args.len(), 1);
        let arg = &schema.positional_args[0];
        assert_eq!(arg.name, "input");
        assert!(arg.required);
        assert_eq!(arg.help.as_deref(), Some("The input file"));
    }

    #[test]
    fn extracts_options_with_defaults_and_possible_values() {
        let cmd = Command::new("testcli")
            .arg(
                Arg::new("output")
                    .long("output")
                    .short('o')
                    .default_value("out.txt")
                    .value_name("FILE")
                    .help("Output file"),
            )
            .arg(
                Arg::new("format")
                    .long("format")
                    .value_parser(["json", "yaml", "toml"])
                    .help("Output format"),
            );

        let schema = extract_schema(&cmd);
        assert_eq!(schema.options.len(), 2);

        let output_arg = schema.options.iter().find(|a| a.name == "output").unwrap();
        assert_eq!(output_arg.short, Some('o'));
        assert_eq!(output_arg.long.as_deref(), Some("output"));
        assert_eq!(output_arg.default_value.as_deref(), Some("out.txt"));
        assert_eq!(output_arg.value_name.as_deref(), Some("FILE"));

        let format_arg = schema.options.iter().find(|a| a.name == "format").unwrap();
        assert_eq!(format_arg.possible_values, vec!["json", "yaml", "toml"]);
    }

    #[test]
    fn extracts_bool_flags() {
        let cmd = Command::new("testcli").arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .action(clap::ArgAction::SetTrue)
                .help("Enable verbose output"),
        );

        let schema = extract_schema(&cmd);
        let verbose = schema.options.iter().find(|a| a.name == "verbose").unwrap();
        assert!(!verbose.takes_value);
        assert_eq!(verbose.short, Some('v'));
    }

    #[test]
    fn extracts_subcommand_summaries() {
        let cmd = Command::new("testcli")
            .subcommand(Command::new("init").about("Initialize project"))
            .subcommand(Command::new("build").about("Build project"));

        let schema = extract_schema(&cmd);
        assert_eq!(schema.subcommands.len(), 2);
        assert_eq!(schema.subcommands[0].name, "init");
        assert_eq!(schema.subcommands[0].about.as_deref(), Some("Initialize project"));
        assert_eq!(schema.subcommands[1].name, "build");
        assert_eq!(schema.subcommands[1].about.as_deref(), Some("Build project"));
    }

    #[test]
    fn extra_description_passed_through() {
        let cmd = Command::new("testcli");
        let schema = extract_schema_with_extra(&cmd, Some("Extra notes here".to_string()));
        assert_eq!(schema.extra_description.as_deref(), Some("Extra notes here"));
    }

    #[test]
    fn hidden_args_excluded() {
        let cmd = Command::new("testcli")
            .arg(
                Arg::new("visible")
                    .long("visible")
                    .help("Visible arg"),
            )
            .arg(
                Arg::new("hidden")
                    .long("hidden")
                    .hide(true)
                    .help("Hidden arg"),
            );

        let schema = extract_schema(&cmd);
        assert_eq!(schema.options.len(), 1);
        assert_eq!(schema.options[0].name, "visible");
    }
}
