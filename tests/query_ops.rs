/// Integration tests for the fme query DSL.
/// Covers every operator with both positive (should match) and negative (should not match) cases.
/// Fixtures: tests/fixtures/query_ops_rich.md  — all field types present
///           tests/fixtures/query_ops_sparse.md — minimal fields (no difficulty, no sr)
use std::process::Command;

fn fme_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_md-fme"))
}

fn q(expr: &str) -> String {
    let output = fme_bin()
        .args(["query", expr, "--folder", "tests/fixtures"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&output.stdout).to_string()
}

// ── Equality (=) ────────────────────────────────────────────────────────────

#[test]
fn eq_string_matches() {
    let out = q("type = query-ops-test");
    assert!(out.contains("query_ops_rich.md"), "eq string: {out}");
}

#[test]
fn eq_case_insensitive() {
    // status = "revisited" but query uses uppercase
    let out = q("status = Revisited");
    assert!(out.contains("query_ops_rich.md"), "eq case-insensitive: {out}");
}

#[test]
fn eq_integer() {
    let out = q("id = 9999");
    assert!(out.contains("query_ops_rich.md"), "eq integer: {out}");
}

#[test]
fn eq_boolean_true() {
    let out = q("study_block = true");
    assert!(out.contains("query_ops_rich.md"), "eq bool true: {out}");
}

// ── Not-equal (!=) ──────────────────────────────────────────────────────────

#[test]
fn neq_excludes_matching_value() {
    let out = q("type != query-ops-test");
    assert!(!out.contains("query_ops_rich.md"), "neq should exclude: {out}");
}

#[test]
fn neq_includes_non_matching_value() {
    let out = q("type != query-ops-sparse");
    assert!(out.contains("query_ops_rich.md"), "neq includes other: {out}");
}

// ── Numeric comparisons ──────────────────────────────────────────────────────

#[test]
fn lt_numeric_matches() {
    let out = q("id < 10000");
    assert!(out.contains("query_ops_rich.md"), "lt numeric: {out}");
}

#[test]
fn lt_numeric_excludes() {
    let out = q("id < 9999");
    assert!(!out.contains("query_ops_rich.md"), "lt numeric excludes: {out}");
}

#[test]
fn lte_numeric_matches_equal() {
    let out = q("id <= 9999");
    assert!(out.contains("query_ops_rich.md"), "lte numeric eq: {out}");
}

#[test]
fn gt_numeric_matches() {
    let out = q("id > 9000");
    assert!(out.contains("query_ops_rich.md"), "gt numeric: {out}");
}

#[test]
fn gt_numeric_excludes_equal() {
    let out = q("id > 9999");
    assert!(!out.contains("query_ops_rich.md"), "gt numeric excludes equal: {out}");
}

#[test]
fn gte_numeric_matches_equal() {
    let out = q("id >= 9999");
    assert!(out.contains("query_ops_rich.md"), "gte numeric eq: {out}");
}

// ── Date comparisons ─────────────────────────────────────────────────────────

#[test]
fn lt_date_matches() {
    // date = "2026-01-15"
    let out = q("date < 2026-02-01");
    assert!(out.contains("query_ops_rich.md"), "lt date: {out}");
}

#[test]
fn lt_date_excludes() {
    let out = q("date < 2026-01-15");
    assert!(!out.contains("query_ops_rich.md"), "lt date excludes: {out}");
}

#[test]
fn lte_date_matches_equal() {
    let out = q("date <= 2026-01-15");
    assert!(out.contains("query_ops_rich.md"), "lte date eq: {out}");
}

#[test]
fn gt_date_matches() {
    let out = q("date > 2026-01-01");
    assert!(out.contains("query_ops_rich.md"), "gt date: {out}");
}

#[test]
fn gt_date_excludes() {
    // date = 2026-01-15, query > 2026-02-01 → should not match
    let out = q("date > 2026-02-01");
    assert!(!out.contains("query_ops_rich.md"), "gt date excludes: {out}");
}

#[test]
fn gte_date_matches_equal() {
    let out = q("date >= 2026-01-15");
    assert!(out.contains("query_ops_rich.md"), "gte date eq: {out}");
}

// ── contains — array ─────────────────────────────────────────────────────────

#[test]
fn contains_array_exact_match() {
    // topics = ["graph", "bfs", "dp"]
    let out = q("topics contains bfs");
    assert!(out.contains("query_ops_rich.md"), "contains array: {out}");
}

#[test]
fn contains_array_case_insensitive() {
    let out = q("topics contains BFS");
    assert!(out.contains("query_ops_rich.md"), "contains array ci: {out}");
}

#[test]
fn contains_array_no_match() {
    let out = q("topics contains zzznomatch");
    assert!(!out.contains("query_ops_rich.md"), "contains array no match: {out}");
}

// ── contains — string substring ──────────────────────────────────────────────

#[test]
fn contains_string_substring_match() {
    // notes = "this is a searchable substring note"
    let out = q("notes contains searchable");
    assert!(out.contains("query_ops_rich.md"), "contains string: {out}");
}

#[test]
fn contains_string_case_insensitive() {
    let out = q("notes contains SEARCHABLE");
    assert!(out.contains("query_ops_rich.md"), "contains string ci: {out}");
}

#[test]
fn contains_string_no_match() {
    let out = q("notes contains zzznomatch");
    assert!(!out.contains("query_ops_rich.md"), "contains string no match: {out}");
}

// ── exists ───────────────────────────────────────────────────────────────────

#[test]
fn exists_present_field() {
    let out = q("sr.next_review exists");
    assert!(out.contains("query_ops_rich.md"), "exists present: {out}");
}

#[test]
fn exists_absent_field_not_matched() {
    // sparse fixture has no sr table
    let out = q("sr.next_review exists");
    assert!(!out.contains("query_ops_sparse.md"), "exists should not match sparse: {out}");
}

#[test]
fn exists_top_level_field() {
    let out = q("difficulty exists");
    assert!(out.contains("query_ops_rich.md"), "exists top-level: {out}");
}

// ── missing ──────────────────────────────────────────────────────────────────

#[test]
fn missing_absent_field_matches() {
    // sparse fixture has no difficulty or sr
    let out = q("difficulty missing");
    assert!(out.contains("query_ops_sparse.md"), "missing absent: {out}");
}

#[test]
fn missing_present_field_not_matched() {
    let out = q("difficulty missing");
    assert!(!out.contains("query_ops_rich.md"), "missing should not match rich: {out}");
}

#[test]
fn missing_nested_absent() {
    let out = q("sr.next_review missing");
    assert!(out.contains("query_ops_sparse.md"), "missing nested absent: {out}");
}

// ── AND ──────────────────────────────────────────────────────────────────────

#[test]
fn and_both_conditions_match() {
    let out = q("type = query-ops-test AND status = revisited");
    assert!(out.contains("query_ops_rich.md"), "and both: {out}");
}

#[test]
fn and_one_condition_fails() {
    let out = q("type = query-ops-test AND status = completed");
    assert!(!out.contains("query_ops_rich.md"), "and one fails: {out}");
}

#[test]
fn and_three_conditions() {
    let out = q("type = query-ops-test AND difficulty = easy AND id = 9999");
    assert!(out.contains("query_ops_rich.md"), "and three: {out}");
}

// ── OR ───────────────────────────────────────────────────────────────────────

#[test]
fn or_first_condition_matches() {
    let out = q("type = query-ops-test OR type = daily-log");
    assert!(out.contains("query_ops_rich.md"), "or first: {out}");
}

#[test]
fn or_second_condition_matches() {
    let out = q("type = nonexistent OR type = query-ops-test");
    assert!(out.contains("query_ops_rich.md"), "or second: {out}");
}

#[test]
fn or_neither_condition_matches() {
    let out = q("type = nonexistent OR type = alsononexistent");
    assert!(!out.contains("query_ops_rich.md"), "or neither: {out}");
}

// ── Parentheses ───────────────────────────────────────────────────────────────

#[test]
fn parens_or_inside_and() {
    let out = q("(status = revisited OR status = completed) AND type = query-ops-test");
    assert!(out.contains("query_ops_rich.md"), "parens or in and: {out}");
}

#[test]
fn parens_wrong_branch_excluded() {
    let out = q("(status = attempted OR status = completed) AND type = query-ops-test");
    assert!(!out.contains("query_ops_rich.md"), "parens wrong branch: {out}");
}

#[test]
fn parens_nested() {
    let out = q("(type = query-ops-test AND (difficulty = easy OR difficulty = medium)) AND id = 9999");
    assert!(out.contains("query_ops_rich.md"), "nested parens: {out}");
}

// ── Nested field access (dot notation) ───────────────────────────────────────

#[test]
fn nested_eq() {
    // sr.interval = 10
    let out = q("sr.interval = 10");
    assert!(out.contains("query_ops_rich.md"), "nested eq: {out}");
}

#[test]
fn nested_gt() {
    let out = q("sr.interval > 5");
    assert!(out.contains("query_ops_rich.md"), "nested gt: {out}");
}

#[test]
fn nested_float_gte() {
    // sr.ease = 2.8
    let out = q("sr.ease >= 2.5");
    assert!(out.contains("query_ops_rich.md"), "nested float gte: {out}");
}

#[test]
fn nested_exists() {
    let out = q("sr.ease exists");
    assert!(out.contains("query_ops_rich.md"), "nested exists: {out}");
}

// ── today magic value ─────────────────────────────────────────────────────────

#[test]
fn today_future_review_gt() {
    // sr.next_review = "2026-03-15" > today (2026-02-27)
    let out = q("sr.next_review > today");
    assert!(out.contains("query_ops_rich.md"), "today future: {out}");
}

#[test]
fn today_past_date_lt() {
    // date = "2026-01-15" < today
    let out = q("date < today");
    assert!(out.contains("query_ops_rich.md"), "today past: {out}");
}

#[test]
fn today_combined_with_and() {
    let out = q("type = query-ops-test AND date < today");
    assert!(out.contains("query_ops_rich.md"), "today + and: {out}");
}
