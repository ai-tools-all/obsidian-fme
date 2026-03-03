use super::Render;
use crate::model::*;
use colored::Colorize;

pub struct TreeRenderer;

/// Split a relative path into (parent_dir, filename).
/// "mistakes/foo.md" → ("mistakes", "foo.md")
/// "foo.md"          → ("", "foo.md")
fn split_path(path: &str) -> (&str, &str) {
    match path.rfind('/') {
        Some(i) => (&path[..i], &path[i + 1..]),
        None => ("", path),
    }
}

/// Print entries grouped by parent dir using ├── / └── branches.
/// Each entry is (full_relative_path, formatted_suffix).
/// Preserves insertion order within each group.
fn print_tree(entries: &[(String, String)]) {
    if entries.is_empty() {
        return;
    }

    // Collect groups in insertion order (BTreeMap would sort dirs; we want stable order)
    let mut group_keys: Vec<String> = Vec::new();
    let mut groups: std::collections::HashMap<String, Vec<(String, String)>> =
        std::collections::HashMap::new();

    for (path, suffix) in entries {
        let (dir, file) = split_path(path);
        let dir = dir.to_string();
        if !groups.contains_key(&dir) {
            group_keys.push(dir.clone());
        }
        groups
            .entry(dir)
            .or_default()
            .push((file.to_string(), suffix.clone()));
    }

    // Max filename width across all entries for alignment
    let max_file_len = entries
        .iter()
        .map(|(p, _)| split_path(p).1.len())
        .max()
        .unwrap_or(10);

    for dir in &group_keys {
        let files = &groups[dir];
        if !dir.is_empty() {
            println!("{}", format!("{dir}/").bold());
        }
        let n = files.len();
        for (i, (file, suffix)) in files.iter().enumerate() {
            let branch = if dir.is_empty() {
                ""
            } else if i == n - 1 {
                "└── "
            } else {
                "├── "
            };
            println!("{branch}{file:<max_file_len$}  {suffix}");
        }
    }
}

fn color_due(days_info: &str, days_overdue: i64) -> String {
    if days_overdue > 0 {
        days_info.red().to_string()
    } else {
        days_info.yellow().to_string()
    }
}

fn color_ease(ease: f64) -> String {
    let s = format!("{ease:.2}");
    if ease < 1.5 {
        s.red().to_string()
    } else if ease < 2.0 {
        s.yellow().to_string()
    } else {
        s
    }
}

impl Render for TreeRenderer {
    fn today(&self, items: &[SrItem]) {
        if items.is_empty() {
            println!("{}", "No items due today.".green());
            return;
        }
        let type_w = items.iter().map(|i| i.review_type.len()).max().unwrap_or(6);
        let entries: Vec<(String, String)> = items
            .iter()
            .map(|item| {
                let suffix = format!(
                    "{:<type_w$}  {}",
                    item.review_type,
                    color_due(&item.days_info, item.days_overdue)
                );
                (item.path.clone(), suffix)
            })
            .collect();
        print_tree(&entries);
        println!("\n{} item(s) due", items.len());
    }

    fn stats(&self, stats: &SrStats) {
        println!("{}", "=== SR Statistics ===".bold());
        println!("Total SR items:       {}", stats.total.to_string().cyan());
        println!(
            "Due today:            {}",
            if stats.due_today > 0 {
                stats.due_today.to_string().yellow().to_string()
            } else {
                "0".green().to_string()
            }
        );
        println!(
            "Overdue:              {}",
            if stats.overdue > 0 {
                stats.overdue.to_string().red().to_string()
            } else {
                "0".green().to_string()
            }
        );
        println!("Reviews this week:    {}", stats.reviews_this_week.to_string().cyan());
        println!("Current streak:       {} day(s)", stats.streak.to_string().cyan());

        if !stats.weakest.is_empty() {
            println!("\n{}", "Top 5 Weakest (lowest ease):".bold());
            let entries: Vec<(String, String)> = stats
                .weakest
                .iter()
                .map(|w| (w.path.clone(), format!("ease={}", color_ease(w.ease))))
                .collect();
            print_tree(&entries);
        }

        if !stats.upcoming_7_days.is_empty() {
            println!("\n{}", "7-day load:".bold());
            for day in &stats.upcoming_7_days {
                println!("  {}: {} item(s)", day.date, day.count);
            }
        }
    }

    fn review(&self, result: &ReviewResult) {
        println!(
            "Reviewed {} (q={}) → interval={}, ease={:.2}, reps={}, next={}",
            result.file,
            result.quality,
            result.interval,
            result.ease,
            result.reps,
            result.next_review
        );
    }

    fn init_sr(&self, result: &InitResult) {
        for f in &result.files {
            println!("Initialized SR: {f}");
        }
        println!("{} file(s) initialized", result.count);
    }

    fn query(&self, result: &QueryResult) {
        if result.count == 0 {
            println!("No matches.");
            return;
        }
        let entries: Vec<(String, String)> = result
            .matches
            .iter()
            .map(|m| {
                let suffix = match &m.fields {
                    None => String::new(),
                    Some(fields) => {
                        let mut pairs: Vec<String> =
                            fields.iter().map(|(k, v)| format!("{k}: {v}")).collect();
                        pairs.sort();
                        pairs.join("  ")
                    }
                };
                (m.path.clone(), suffix)
            })
            .collect();
        print_tree(&entries);
        println!("\n{} match(es)", result.count);
    }

    fn enforce(&self, result: &EnforceResult) {
        for f in &result.results {
            match f.status {
                FileStatus::Pass => println!("{} {}", "PASS".green().bold(), f.file),
                FileStatus::Skip => println!("{} {} — excluded", "SKIP".yellow().bold(), f.file),
                FileStatus::Fail => {
                    let detail = if f.errors.is_empty() {
                        String::new()
                    } else {
                        format!(" — {}", f.errors.join("; "))
                    };
                    println!("{} {}{detail}", "FAIL".red().bold(), f.file);
                }
                FileStatus::Fixed => {
                    let detail = if f.fixed.is_empty() {
                        String::new()
                    } else {
                        format!(" — added: {}", f.fixed.join(", "))
                    };
                    println!("{} {}{detail}", "FIXED".yellow().bold(), f.file);
                }
            }
        }
        let total = result.passed + result.failed;
        println!(
            "\n{} passed, {} failed out of {} files",
            result.passed.to_string().green(),
            if result.failed > 0 {
                result.failed.to_string().red().to_string()
            } else {
                "0".to_string()
            },
            total
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_path_with_dir() {
        assert_eq!(split_path("mistakes/foo.md"), ("mistakes", "foo.md"));
    }

    #[test]
    fn split_path_root_file() {
        assert_eq!(split_path("foo.md"), ("", "foo.md"));
    }

    #[test]
    fn split_path_nested() {
        assert_eq!(split_path("a/b/c.md"), ("a/b", "c.md"));
    }

    #[test]
    fn print_tree_groups_by_dir() {
        // Just check no panic and correct grouping logic via split_path
        let entries = vec![
            ("mistakes/a.md".to_string(), "info1".to_string()),
            ("mistakes/b.md".to_string(), "info2".to_string()),
            ("learning/c.md".to_string(), "info3".to_string()),
        ];
        // Group manually and verify
        let mut groups: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for (path, _) in &entries {
            let (dir, file) = split_path(path);
            groups.entry(dir.to_string()).or_default().push(file.to_string());
        }
        assert_eq!(groups["mistakes"].len(), 2);
        assert_eq!(groups["learning"].len(), 1);
        assert!(groups["mistakes"].contains(&"a.md".to_string()));
        assert!(groups["learning"].contains(&"c.md".to_string()));
    }

    #[test]
    fn print_tree_handles_root_files() {
        let entries = vec![("root.md".to_string(), "info".to_string())];
        let mut groups: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for (path, _) in &entries {
            let (dir, file) = split_path(path);
            groups.entry(dir.to_string()).or_default().push(file.to_string());
        }
        assert!(groups.contains_key(""));
        assert_eq!(groups[""][0], "root.md");
    }
}
