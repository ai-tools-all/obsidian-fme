use super::Render;
use crate::model::*;
use serde::Serialize;

pub struct JsonRenderer;

fn emit<T: Serialize>(val: &T) {
    println!("{}", serde_json::to_string_pretty(val).expect("JSON serialization failed"));
}

#[derive(Serialize)]
struct TodayJson<'a> {
    total: usize,
    items: &'a [SrItem],
}

#[derive(Serialize)]
struct ReviewJson<'a> {
    status: &'static str,
    #[serde(flatten)]
    result: &'a ReviewResult,
}

#[derive(Serialize)]
struct InitJson<'a> {
    status: &'static str,
    #[serde(flatten)]
    result: &'a InitResult,
}

#[cfg(test)]
pub fn format_today(items: &[SrItem]) -> String {
    serde_json::to_string_pretty(&TodayJson { total: items.len(), items })
        .expect("JSON serialization failed")
}

#[cfg(test)]
pub fn format_stats(stats: &SrStats) -> String {
    serde_json::to_string_pretty(stats).expect("JSON serialization failed")
}

#[cfg(test)]
pub fn format_review(result: &ReviewResult) -> String {
    serde_json::to_string_pretty(&ReviewJson { status: "ok", result })
        .expect("JSON serialization failed")
}

#[cfg(test)]
pub fn format_init_sr(result: &InitResult) -> String {
    serde_json::to_string_pretty(&InitJson { status: "ok", result })
        .expect("JSON serialization failed")
}

#[cfg(test)]
pub fn format_query(result: &QueryResult) -> String {
    serde_json::to_string_pretty(result).expect("JSON serialization failed")
}

#[cfg(test)]
pub fn format_enforce(result: &EnforceResult) -> String {
    serde_json::to_string_pretty(result).expect("JSON serialization failed")
}

impl Render for JsonRenderer {
    fn today(&self, items: &[SrItem]) {
        emit(&TodayJson { total: items.len(), items });
    }
    fn stats(&self, stats: &SrStats) {
        emit(stats);
    }
    fn review(&self, result: &ReviewResult) {
        emit(&ReviewJson { status: "ok", result });
    }
    fn init_sr(&self, result: &InitResult) {
        emit(&InitJson { status: "ok", result });
    }
    fn query(&self, result: &QueryResult) {
        emit(result);
    }
    fn enforce(&self, result: &EnforceResult) {
        emit(result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sr_item(path: &str, overdue: i64) -> SrItem {
        SrItem {
            path: path.into(),
            review_type: "recall".into(),
            days_overdue: overdue,
            days_info: if overdue == 0 { "today".into() } else { format!("{overdue} day(s) overdue") },
            ease: 2.5,
            next_review: "2026-03-03".into(),
        }
    }

    #[test]
    fn today_json_has_total_and_items() {
        let items = vec![make_sr_item("mistakes/a.md", 0), make_sr_item("mistakes/b.md", 3)];
        let out = format_today(&items);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["total"], 2);
        assert_eq!(v["items"].as_array().unwrap().len(), 2);
        assert_eq!(v["items"][0]["path"], "mistakes/a.md");
        assert_eq!(v["items"][1]["days_overdue"], 3);
    }

    #[test]
    fn today_json_empty() {
        let out = format_today(&[]);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["total"], 0);
        assert_eq!(v["items"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn stats_json_structure() {
        let stats = SrStats {
            total: 10,
            due_today: 3,
            overdue: 2,
            reviews_this_week: 5,
            streak: 4,
            weakest: vec![WeakItem { path: "a.md".into(), ease: 1.8 }],
            upcoming_7_days: vec![UpcomingDay { date: "2026-03-04".into(), count: 1 }],
        };
        let out = format_stats(&stats);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["total"], 10);
        assert_eq!(v["due_today"], 3);
        assert_eq!(v["streak"], 4);
        assert_eq!(v["weakest"][0]["ease"], 1.8);
        assert_eq!(v["upcoming_7_days"][0]["count"], 1);
    }

    #[test]
    fn review_json_has_status_ok() {
        let r = ReviewResult {
            file: "mistakes/a.md".into(),
            quality: 4,
            interval: 7,
            ease: 2.6,
            reps: 3,
            next_review: "2026-03-10".into(),
        };
        let out = format_review(&r);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["status"], "ok");
        assert_eq!(v["quality"], 4);
        assert_eq!(v["interval"], 7);
        assert_eq!(v["next_review"], "2026-03-10");
    }

    #[test]
    fn init_sr_json_has_status_ok() {
        let r = InitResult { count: 2, files: vec!["a.md".into(), "b.md".into()] };
        let out = format_init_sr(&r);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["status"], "ok");
        assert_eq!(v["count"], 2);
        assert_eq!(v["files"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn query_json_structure() {
        let r = QueryResult {
            count: 1,
            matches: vec![QueryMatch { path: "mistakes/a.md".into(), fields: None }],
        };
        let out = format_query(&r);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["count"], 1);
        assert_eq!(v["matches"][0]["path"], "mistakes/a.md");
    }

    #[test]
    fn query_json_with_fields() {
        let mut fields = std::collections::HashMap::new();
        fields.insert("status".into(), "completed".into());
        let r = QueryResult {
            count: 1,
            matches: vec![QueryMatch { path: "a.md".into(), fields: Some(fields) }],
        };
        let out = format_query(&r);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["matches"][0]["fields"]["status"], "completed");
    }

    #[test]
    fn enforce_json_structure() {
        let r = EnforceResult {
            passed: 2,
            failed: 1,
            results: vec![
                EnforceFile { file: "a.md".into(), status: FileStatus::Pass, errors: vec![], fixed: vec![] },
                EnforceFile { file: "b.md".into(), status: FileStatus::Fail, errors: vec!["missing: status".into()], fixed: vec![] },
            ],
        };
        let out = format_enforce(&r);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["passed"], 2);
        assert_eq!(v["failed"], 1);
        assert_eq!(v["results"][0]["status"], "pass");
        assert_eq!(v["results"][1]["status"], "fail");
        assert_eq!(v["results"][1]["errors"][0], "missing: status");
    }
}
