use colored::Colorize;

pub struct TableRow {
    pub file: String,
    pub review_type: String,
    pub days_info: String,
}

pub fn print_today_table(rows: &[TableRow]) {
    if rows.is_empty() {
        println!("{}", "No items due today.".green());
        return;
    }
    let fw = rows.iter().map(|r| r.file.len()).max().unwrap_or(20).max(4);
    let rw = rows
        .iter()
        .map(|r| r.review_type.len())
        .max()
        .unwrap_or(10)
        .max(4);

    println!(
        "{:<fw$}  {:<rw$}  {}",
        "File".bold(),
        "Type".bold(),
        "Due".bold(),
        fw = fw,
        rw = rw
    );
    println!("{}", "-".repeat(fw + rw + 20));

    for row in rows {
        let due_colored = if row.days_info.contains("overdue") {
            row.days_info.red().to_string()
        } else if row.days_info.contains("today") {
            row.days_info.yellow().to_string()
        } else {
            row.days_info.green().to_string()
        };
        println!(
            "{:<fw$}  {:<rw$}  {}",
            row.file,
            row.review_type,
            due_colored,
            fw = fw,
            rw = rw
        );
    }
    println!("\n{} item(s) due", rows.len());
}

pub fn print_stats(
    total: usize,
    due_today: usize,
    overdue: usize,
    reviews_this_week: usize,
    streak: usize,
    weakest: &[(String, f64)],
    upcoming: &[(String, usize)],
) {
    println!("{}", "=== SR Statistics ===".bold());
    println!("Total SR items:       {}", total.to_string().cyan());
    println!("Due today:            {}", format_count(due_today));
    println!("Overdue:              {}", format_overdue(overdue));
    println!("Reviews this week:    {}", reviews_this_week.to_string().cyan());
    println!("Current streak:       {} day(s)", streak.to_string().cyan());

    if !weakest.is_empty() {
        println!("\n{}", "Top 5 Weakest (lowest ease):".bold());
        for (name, ease) in weakest {
            let ease_str = format!("{ease:.2}");
            let colored = if *ease < 1.5 {
                ease_str.red().to_string()
            } else if *ease < 2.0 {
                ease_str.yellow().to_string()
            } else {
                ease_str.to_string()
            };
            println!("  {name}  ease={colored}");
        }
    }

    if !upcoming.is_empty() {
        println!("\n{}", "7-day load:".bold());
        for (day, count) in upcoming {
            println!("  {day}: {count} item(s)");
        }
    }
}

fn format_count(n: usize) -> String {
    if n > 0 {
        n.to_string().yellow().to_string()
    } else {
        "0".green().to_string()
    }
}

fn format_overdue(n: usize) -> String {
    if n > 0 {
        n.to_string().red().to_string()
    } else {
        "0".green().to_string()
    }
}
