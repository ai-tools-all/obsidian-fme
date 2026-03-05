use crate::model::{ArgSchema, CommandSchema};

fn format_flag(arg: &ArgSchema) -> String {
    let mut parts = Vec::new();
    if let Some(short) = arg.short {
        parts.push(format!("-{short}"));
    }
    if let Some(ref long) = arg.long {
        parts.push(format!("--{long}"));
    }
    let flag = parts.join(", ");
    if arg.takes_value {
        if let Some(ref vn) = arg.value_name {
            format!("{flag} <{vn}>")
        } else {
            format!("{flag} <{}>", arg.name.to_uppercase())
        }
    } else {
        flag
    }
}

impl CommandSchema {
    pub fn to_markdown(&self) -> String {
        let mut out = String::new();

        out.push_str(&format!("# `{}`\n\n", self.name));

        let description = self
            .long_about
            .as_deref()
            .or(self.about.as_deref());
        if let Some(desc) = description {
            out.push_str(desc);
            out.push_str("\n\n");
        }

        out.push_str(&format!("**Usage:** `{}`\n", self.usage));

        if !self.positional_args.is_empty() {
            out.push_str("\n## Arguments\n\n");
            out.push_str("| Name | Required | Description |\n");
            out.push_str("|------|----------|-------------|\n");
            for arg in &self.positional_args {
                let name = arg
                    .value_name
                    .as_deref()
                    .unwrap_or(&arg.name)
                    .to_uppercase();
                let required = if arg.required { "yes" } else { "no" };
                let help = arg.help.as_deref().unwrap_or("-");
                out.push_str(&format!("| `{name}` | {required} | {help} |\n"));
            }
        }

        if !self.options.is_empty() {
            out.push_str("\n## Options\n\n");
            out.push_str(
                "| Flag | Required | Default | Possible Values | Description |\n",
            );
            out.push_str(
                "|------|----------|---------|-----------------|-------------|\n",
            );
            for arg in &self.options {
                let flag = format_flag(arg);
                let required = if arg.required { "yes" } else { "no" };
                let default = arg.default_value.as_deref().unwrap_or("-");
                let possible = if arg.possible_values.is_empty() {
                    "-".to_string()
                } else {
                    arg.possible_values.join(", ")
                };
                let help = arg.help.as_deref().unwrap_or("-");
                out.push_str(&format!(
                    "| `{flag}` | {required} | {default} | {possible} | {help} |\n"
                ));
            }
        }

        if !self.subcommands.is_empty() {
            out.push_str("\n## Subcommands\n\n");
            for sub in &self.subcommands {
                let about = sub.about.as_deref().unwrap_or("");
                if about.is_empty() {
                    out.push_str(&format!("- `{}`\n", sub.name));
                } else {
                    out.push_str(&format!("- `{}` — {}\n", sub.name, about));
                }
            }
        }

        if let Some(ref extra) = self.extra_description {
            out.push_str("\n## Notes\n\n");
            out.push_str(extra);
            out.push('\n');
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use crate::model::*;

    fn minimal_schema() -> CommandSchema {
        CommandSchema {
            name: "mycli".to_string(),
            about: Some("Short about".to_string()),
            long_about: None,
            usage: "mycli [OPTIONS]".to_string(),
            positional_args: vec![],
            options: vec![],
            subcommands: vec![],
            extra_description: None,
        }
    }

    #[test]
    fn markdown_includes_command_name_and_about() {
        let md = minimal_schema().to_markdown();
        assert!(md.contains("# `mycli`"));
        assert!(md.contains("Short about"));
        assert!(md.contains("**Usage:** `mycli [OPTIONS]`"));
    }

    #[test]
    fn markdown_renders_options_table() {
        let mut schema = minimal_schema();
        schema.options = vec![ArgSchema {
            name: "folder".to_string(),
            short: Some('f'),
            long: Some("folder".to_string()),
            required: false,
            default_value: Some(".".to_string()),
            possible_values: vec![],
            help: Some("Folder to scan".to_string()),
            value_name: Some("FOLDER".to_string()),
            takes_value: true,
        }];

        let md = schema.to_markdown();
        assert!(md.contains("## Options"));
        assert!(md.contains("| `-f, --folder <FOLDER>` | no | . | - | Folder to scan |"));
    }

    #[test]
    fn markdown_renders_extra_description() {
        let mut schema = minimal_schema();
        schema.extra_description = Some("Extra notes for agents".to_string());

        let md = schema.to_markdown();
        assert!(md.contains("## Notes"));
        assert!(md.contains("Extra notes for agents"));
    }

    #[test]
    fn markdown_prefers_long_about_over_about() {
        let mut schema = minimal_schema();
        schema.about = Some("Short".to_string());
        schema.long_about = Some("Long detailed description".to_string());

        let md = schema.to_markdown();
        assert!(md.contains("Long detailed description"));
        assert!(!md.contains("\nShort\n"));
    }
}
