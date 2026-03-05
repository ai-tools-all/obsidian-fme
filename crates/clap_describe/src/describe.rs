use clap::Args;

use crate::extract::extract_schema_with_extra;

#[derive(Debug, Clone, Args)]
pub struct Describe {
    /// Print a structured description of this command (for agents/LLMs)
    #[arg(long)]
    pub describe: bool,
}

impl Describe {
    pub fn handle(
        &self,
        cmd: &clap::Command,
        json: bool,
        extra: Option<String>,
    ) -> Option<String> {
        if !self.describe {
            return None;
        }
        let schema = extract_schema_with_extra(cmd, extra);
        if json {
            Some(serde_json::to_string_pretty(&schema).unwrap())
        } else {
            Some(schema.to_markdown())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Command;

    fn test_cmd() -> Command {
        Command::new("testcli").about("A test tool")
    }

    #[test]
    fn handle_returns_none_when_not_set() {
        let d = Describe { describe: false };
        assert!(d.handle(&test_cmd(), false, None).is_none());
    }

    #[test]
    fn handle_returns_markdown_by_default() {
        let d = Describe { describe: true };
        let result = d.handle(&test_cmd(), false, None).unwrap();
        assert!(result.contains("# `testcli`"));
        assert!(result.contains("A test tool"));
    }

    #[test]
    fn handle_returns_json_when_requested() {
        let d = Describe { describe: true };
        let result = d.handle(&test_cmd(), true, None).unwrap();
        let val: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(val["name"], "testcli");
        assert_eq!(val["about"], "A test tool");
    }

    #[test]
    fn handle_passes_extra_description() {
        let d = Describe { describe: true };
        let result = d
            .handle(&test_cmd(), false, Some("Extra info".to_string()))
            .unwrap();
        assert!(result.contains("Extra info"));
    }
}
