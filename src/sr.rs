use crate::{display, frontmatter};
use chrono::{Datelike, Local, NaiveDate};
use std::path::Path;
use toml::Value;

fn today_date() -> NaiveDate {
    Local::now().date_naive()
}

fn parse_sr_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

struct SrData {
    next_review: NaiveDate,
    interval: f64,
    ease: f64,
    reps: u32,
    last_reviewed: Option<NaiveDate>,
}

fn extract_sr(fm: &Value) -> Option<SrData> {
    let sr = fm.get("sr")?;
    let next_review = parse_sr_date(&frontmatter::value_to_string(sr.get("next_review")?))?;
    let interval = sr
        .get("interval")
        .and_then(|v| frontmatter::value_to_string(v).parse::<f64>().ok())
        .unwrap_or(1.0);
    let ease = sr
        .get("ease")
        .and_then(|v| frontmatter::value_to_string(v).parse::<f64>().ok())
        .unwrap_or(2.5);
    let reps = sr
        .get("reps")
        .and_then(|v| frontmatter::value_to_string(v).parse::<u32>().ok())
        .unwrap_or(0);
    let last_reviewed = sr.get("last_reviewed").and_then(|v| {
        let s = frontmatter::value_to_string(v);
        if s == "~" || s == "null" {
            None
        } else {
            parse_sr_date(&s)
        }
    });
    Some(SrData {
        next_review,
        interval,
        ease,
        reps,
        last_reviewed,
    })
}

fn sm2(quality: u8, sr: &SrData) -> (f64, f64, u32) {
    let q = quality as f64;
    let new_ease = (sr.ease + (0.1 - (5.0 - q) * (0.08 + (5.0 - q) * 0.02))).max(1.3);

    if quality < 3 {
        (1.0, new_ease, 0)
    } else {
        let new_reps = sr.reps + 1;
        let new_interval = if sr.reps == 0 {
            1.0
        } else if sr.reps == 1 {
            6.0
        } else if quality == 3 {
            (sr.interval * new_ease * 0.8).max(1.0)
        } else {
            (sr.interval * new_ease).max(1.0)
        };
        (new_interval, new_ease, new_reps)
    }
}

fn build_sr_table(next: &str, interval: i64, ease: f64, reps: u32, last_reviewed: &str) -> Value {
    let mut sr = toml::map::Map::new();
    sr.insert("next_review".into(), Value::String(next.into()));
    sr.insert("interval".into(), Value::Integer(interval));
    sr.insert("ease".into(), Value::Float(ease));
    sr.insert("reps".into(), Value::Integer(reps as i64));
    sr.insert("last_reviewed".into(), Value::String(last_reviewed.into()));
    Value::Table(sr)
}

pub fn review(file: &Path, quality: u8) -> Result<(), String> {
    if quality > 5 {
        return Err("Quality must be 0-5".into());
    }
    let doc = frontmatter::read_file(file)?;
    let sr = extract_sr(&doc.frontmatter)
        .ok_or_else(|| format!("No sr: block in {}. Run init-sr first.", file.display()))?;

    let (interval, ease, reps) = sm2(quality, &sr);
    let today = today_date();
    let next = today + chrono::Duration::days(interval.round() as i64);

    let mut fm = doc.frontmatter.clone();
    let sr_table = build_sr_table(
        &next.format("%Y-%m-%d").to_string(),
        interval.round() as i64,
        ease,
        reps,
        &today.format("%Y-%m-%d").to_string(),
    );
    fm.as_table_mut()
        .ok_or("Frontmatter is not a table")?
        .insert("sr".into(), sr_table);

    let new_raw = frontmatter::replace_frontmatter(&doc.raw, &fm);
    frontmatter::write_raw(file, &new_raw)?;

    println!(
        "Reviewed {} (q={quality}) → interval={}, ease={:.2}, reps={reps}, next={}",
        file.display(),
        interval.round() as i64,
        ease,
        next.format("%Y-%m-%d")
    );
    Ok(())
}

pub fn init_sr(file: Option<&Path>, folder: Option<&Path>, review_type: &str, depth: usize) -> Result<(), String> {
    if file.is_none() && folder.is_none() {
        return Err("Provide --file or --folder".into());
    }

    let files: Vec<std::path::PathBuf> = if let Some(f) = file {
        vec![f.to_path_buf()]
    } else {
        frontmatter::collect_md_files(folder.unwrap(), depth)
    };

    let today = today_date().format("%Y-%m-%d").to_string();
    let mut count = 0;

    for f in &files {
        let raw = match frontmatter::read_raw(f) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Warning: skipping {}: {e}", f.display());
                continue;
            }
        };

        let doc = match frontmatter::parse(&raw) {
            Some(d) => d,
            None => continue,
        };

        if frontmatter::has_sr_block(&doc.frontmatter) {
            continue;
        }

        let mut fm = doc.frontmatter.clone();
        let table = fm.as_table_mut().unwrap();

        let sr_table = build_sr_table(&today, 1, 2.50, 0, "~");
        table.insert("sr".into(), sr_table);

        if !table.contains_key("review_type") {
            table.insert("review_type".into(), Value::String(review_type.into()));
        }

        let new_raw = frontmatter::replace_frontmatter(&raw, &fm);
        frontmatter::write_raw(f, &new_raw)?;
        count += 1;
        println!("Initialized SR: {}", f.display());
    }

    println!("{count} file(s) initialized");
    Ok(())
}

pub fn today(folder: &Path, depth: usize) -> Result<(), String> {
    let files = frontmatter::collect_md_files(folder, depth);
    let today = today_date();
    let mut rows: Vec<(i64, display::TableRow)> = Vec::new();

    for file in &files {
        let doc = match frontmatter::read_file(file) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let sr = match extract_sr(&doc.frontmatter) {
            Some(s) => s,
            None => continue,
        };
        if sr.next_review > today {
            continue;
        }
        let days_diff = (today - sr.next_review).num_days();
        let days_info = if days_diff == 0 {
            "today".to_string()
        } else {
            format!("{days_diff} day(s) overdue")
        };
        let review_type = doc
            .frontmatter
            .get("review_type")
            .map(frontmatter::value_to_string)
            .unwrap_or_else(|| "recall".into());
        let fname = file.strip_prefix(folder).unwrap_or(file).to_string_lossy().to_string();
        rows.push((
            days_diff,
            display::TableRow {
                file: fname,
                review_type,
                days_info,
            },
        ));
    }

    rows.sort_by_key(|r| std::cmp::Reverse(r.0));
    let table_rows: Vec<display::TableRow> = rows.into_iter().map(|(_, r)| r).collect();
    display::print_today_table(&table_rows);
    Ok(())
}

pub fn stats(folder: &Path, depth: usize) -> Result<(), String> {
    let files = frontmatter::collect_md_files(folder, depth);
    let today = today_date();
    let week_start = today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);

    let mut total = 0usize;
    let mut due_today = 0usize;
    let mut overdue = 0usize;
    let mut reviews_this_week = 0usize;
    let mut review_dates: Vec<NaiveDate> = Vec::new();
    let mut weakest: Vec<(String, f64)> = Vec::new();
    let mut upcoming_map: std::collections::HashMap<NaiveDate, usize> = std::collections::HashMap::new();

    for file in &files {
        let doc = match frontmatter::read_file(file) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let sr = match extract_sr(&doc.frontmatter) {
            Some(s) => s,
            None => continue,
        };
        total += 1;
        let fname = file.strip_prefix(folder).unwrap_or(file).to_string_lossy().to_string();

        let days_diff = (today - sr.next_review).num_days();
        if days_diff >= 0 {
            due_today += 1;
        }
        if days_diff > 0 {
            overdue += 1;
        }

        if let Some(lr) = sr.last_reviewed {
            if lr >= week_start {
                reviews_this_week += 1;
            }
            review_dates.push(lr);
        }

        weakest.push((fname, sr.ease));

        if sr.next_review > today && sr.next_review <= today + chrono::Duration::days(7) {
            *upcoming_map.entry(sr.next_review).or_insert(0) += 1;
        }
    }

    review_dates.sort();
    review_dates.dedup();
    let mut streak = 0usize;
    let mut check = today;
    if !review_dates.contains(&today) {
        check = today - chrono::Duration::days(1);
    }
    loop {
        if review_dates.contains(&check) {
            streak += 1;
            check -= chrono::Duration::days(1);
        } else {
            break;
        }
    }

    weakest.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    weakest.truncate(5);

    let mut upcoming: Vec<(String, usize)> = Vec::new();
    for i in 0..7 {
        let d = today + chrono::Duration::days(i + 1);
        let count = upcoming_map.get(&d).copied().unwrap_or(0);
        upcoming.push((d.format("%Y-%m-%d").to_string(), count));
    }

    display::print_stats(
        total,
        due_today,
        overdue,
        reviews_this_week,
        streak,
        &weakest,
        &upcoming,
    );
    Ok(())
}
