use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct SrItem {
    pub path: String,
    pub review_type: String,
    pub days_overdue: i64,
    pub days_info: String,
    pub ease: f64,
    pub next_review: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WeakItem {
    pub path: String,
    pub ease: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpcomingDay {
    pub date: String,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct SrStats {
    pub total: usize,
    pub due_today: usize,
    pub overdue: usize,
    pub reviews_this_week: usize,
    pub streak: usize,
    pub weakest: Vec<WeakItem>,
    pub upcoming_7_days: Vec<UpcomingDay>,
}

#[derive(Debug, Serialize)]
pub struct ReviewResult {
    pub file: String,
    pub quality: u8,
    pub interval: i64,
    pub ease: f64,
    pub reps: u32,
    pub next_review: String,
}

#[derive(Debug, Serialize)]
pub struct InitResult {
    pub count: usize,
    pub files: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct QueryMatch {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct QueryResult {
    pub count: usize,
    pub matches: Vec<QueryMatch>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FileStatus {
    Pass,
    Fail,
    Fixed,
    Skip,
}

#[derive(Debug, Serialize)]
pub struct EnforceFile {
    pub file: String,
    pub status: FileStatus,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fixed: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct EnforceResult {
    pub passed: u32,
    pub failed: u32,
    pub results: Vec<EnforceFile>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sr_item_serializes_correctly() {
        let item = SrItem {
            path: "mistakes/133_clone_graph.md".into(),
            review_type: "recall".into(),
            days_overdue: 5,
            days_info: "5 day(s) overdue".into(),
            ease: 2.5,
            next_review: "2026-02-26".into(),
        };
        let v: serde_json::Value = serde_json::from_str(&serde_json::to_string(&item).unwrap()).unwrap();
        assert_eq!(v["path"], "mistakes/133_clone_graph.md");
        assert_eq!(v["review_type"], "recall");
        assert_eq!(v["days_overdue"], 5);
        assert_eq!(v["ease"], 2.5);
    }

    #[test]
    fn sr_stats_serializes_correctly() {
        let stats = SrStats {
            total: 27,
            due_today: 5,
            overdue: 3,
            reviews_this_week: 2,
            streak: 1,
            weakest: vec![WeakItem { path: "a.md".into(), ease: 1.8 }],
            upcoming_7_days: vec![UpcomingDay { date: "2026-03-04".into(), count: 2 }],
        };
        let v: serde_json::Value = serde_json::from_str(&serde_json::to_string(&stats).unwrap()).unwrap();
        assert_eq!(v["total"], 27);
        assert_eq!(v["streak"], 1);
        assert_eq!(v["weakest"][0]["ease"], 1.8);
        assert_eq!(v["upcoming_7_days"][0]["date"], "2026-03-04");
    }

    #[test]
    fn enforce_file_status_serializes_lowercase() {
        let f = EnforceFile { file: "a.md".into(), status: FileStatus::Pass, errors: vec![], fixed: vec![] };
        let v: serde_json::Value = serde_json::from_str(&serde_json::to_string(&f).unwrap()).unwrap();
        assert_eq!(v["status"], "pass");
    }

    #[test]
    fn enforce_file_omits_empty_vecs() {
        let f = EnforceFile { file: "a.md".into(), status: FileStatus::Pass, errors: vec![], fixed: vec![] };
        let v: serde_json::Value = serde_json::from_str(&serde_json::to_string(&f).unwrap()).unwrap();
        assert!(v.get("errors").is_none());
        assert!(v.get("fixed").is_none());
    }

    #[test]
    fn query_match_omits_none_fields() {
        let m = QueryMatch { path: "a.md".into(), fields: None };
        let v: serde_json::Value = serde_json::from_str(&serde_json::to_string(&m).unwrap()).unwrap();
        assert!(v.get("fields").is_none());
    }

    #[test]
    fn review_result_serializes_correctly() {
        let r = ReviewResult { file: "a.md".into(), quality: 4, interval: 7, ease: 2.6, reps: 3, next_review: "2026-03-10".into() };
        let v: serde_json::Value = serde_json::from_str(&serde_json::to_string(&r).unwrap()).unwrap();
        assert_eq!(v["quality"], 4);
        assert_eq!(v["interval"], 7);
        assert_eq!(v["next_review"], "2026-03-10");
    }
}
