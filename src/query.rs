use crate::frontmatter;
use chrono::Local;
use std::path::Path;
use toml::Value;

#[derive(Debug, Clone)]
enum Token {
    Field(String),
    Op(Operator),
    Value(String),
    And,
    Or,
    LParen,
    RParen,
}

#[derive(Debug, Clone)]
enum Operator {
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
    Contains,
    Exists,
    Missing,
}

#[derive(Debug)]
enum Expr {
    Condition {
        field: String,
        op: Operator,
        value: Option<String>,
    },
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        if c == '(' {
            tokens.push(Token::LParen);
            chars.next();
            continue;
        }
        if c == ')' {
            tokens.push(Token::RParen);
            chars.next();
            continue;
        }

        let mut word = String::new();
        if c == '<' || c == '>' || c == '!' || c == '=' {
            while let Some(&nc) = chars.peek() {
                if nc == '<' || nc == '>' || nc == '!' || nc == '=' {
                    word.push(nc);
                    chars.next();
                } else {
                    break;
                }
            }
        } else if c == '"' {
            chars.next();
            while let Some(&nc) = chars.peek() {
                if nc == '"' {
                    chars.next();
                    break;
                }
                word.push(nc);
                chars.next();
            }
            tokens.push(Token::Value(word));
            continue;
        } else {
            while let Some(&nc) = chars.peek() {
                if nc.is_whitespace() || nc == '(' || nc == ')' {
                    break;
                }
                word.push(nc);
                chars.next();
            }
        }

        match word.as_str() {
            "AND" | "and" => tokens.push(Token::And),
            "OR" | "or" => tokens.push(Token::Or),
            "=" | "==" => tokens.push(Token::Op(Operator::Eq)),
            "!=" => tokens.push(Token::Op(Operator::Neq)),
            "<" => tokens.push(Token::Op(Operator::Lt)),
            "<=" => tokens.push(Token::Op(Operator::Lte)),
            ">" => tokens.push(Token::Op(Operator::Gt)),
            ">=" => tokens.push(Token::Op(Operator::Gte)),
            "contains" => tokens.push(Token::Op(Operator::Contains)),
            "exists" => tokens.push(Token::Op(Operator::Exists)),
            "missing" => tokens.push(Token::Op(Operator::Missing)),
            _ => {
                if tokens.is_empty() || matches!(tokens.last(), Some(Token::And | Token::Or | Token::LParen)) {
                    tokens.push(Token::Field(word));
                } else {
                    tokens.push(Token::Value(resolve_magic(&word)));
                }
            }
        }
    }
    Ok(tokens)
}

fn resolve_magic(s: &str) -> String {
    if s == "today" {
        Local::now().format("%Y-%m-%d").to_string()
    } else {
        s.to_string()
    }
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        t
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        let left = self.parse_and()?;
        if matches!(self.peek(), Some(Token::Or)) {
            self.next();
            let right = self.parse_expr()?;
            Ok(Expr::Or(Box::new(left), Box::new(right)))
        } else {
            Ok(left)
        }
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let left = self.parse_atom()?;
        if matches!(self.peek(), Some(Token::And)) {
            self.next();
            let right = self.parse_and()?;
            Ok(Expr::And(Box::new(left), Box::new(right)))
        } else {
            Ok(left)
        }
    }

    fn parse_atom(&mut self) -> Result<Expr, String> {
        if matches!(self.peek(), Some(Token::LParen)) {
            self.next();
            let expr = self.parse_expr()?;
            if !matches!(self.peek(), Some(Token::RParen)) {
                return Err("Expected ')'".to_string());
            }
            self.next();
            return Ok(expr);
        }

        let field = match self.next() {
            Some(Token::Field(f)) => f,
            other => return Err(format!("Expected field name, got {other:?}")),
        };

        let op = match self.next() {
            Some(Token::Op(op)) => op,
            other => return Err(format!("Expected operator after `{field}`, got {other:?}")),
        };

        match op {
            Operator::Exists | Operator::Missing => Ok(Expr::Condition {
                field,
                op,
                value: None,
            }),
            _ => {
                let value = match self.next() {
                    Some(Token::Value(v)) => v,
                    Some(Token::Field(v)) => resolve_magic(&v),
                    other => return Err(format!("Expected value, got {other:?}")),
                };
                Ok(Expr::Condition {
                    field,
                    op,
                    value: Some(value),
                })
            }
        }
    }
}

fn evaluate(expr: &Expr, fm: &Value) -> bool {
    match expr {
        Expr::And(a, b) => evaluate(a, fm) && evaluate(b, fm),
        Expr::Or(a, b) => evaluate(a, fm) || evaluate(b, fm),
        Expr::Condition { field, op, value } => {
            let fv = frontmatter::get_nested(fm, field);
            match op {
                Operator::Exists => fv.is_some(),
                Operator::Missing => fv.is_none(),
                _ => {
                    let val = value.as_deref().unwrap_or("");
                    let Some(fv) = fv else { return false };
                    match op {
                        Operator::Contains => contains_check(fv, val),
                        Operator::Eq => eq_check(fv, val),
                        Operator::Neq => !eq_check(fv, val),
                        Operator::Lt => cmp_check(fv, val) == Some(std::cmp::Ordering::Less),
                        Operator::Lte => matches!(cmp_check(fv, val), Some(std::cmp::Ordering::Less | std::cmp::Ordering::Equal)),
                        Operator::Gt => cmp_check(fv, val) == Some(std::cmp::Ordering::Greater),
                        Operator::Gte => matches!(cmp_check(fv, val), Some(std::cmp::Ordering::Greater | std::cmp::Ordering::Equal)),
                        _ => false,
                    }
                }
            }
        }
    }
}

fn contains_check(fv: &Value, val: &str) -> bool {
    match fv {
        Value::Array(arr) => arr.iter().any(|item| {
            frontmatter::value_to_string(item).eq_ignore_ascii_case(val)
        }),
        Value::String(s) => s.to_lowercase().contains(&val.to_lowercase()),
        _ => frontmatter::value_to_string(fv)
            .to_lowercase()
            .contains(&val.to_lowercase()),
    }
}

fn eq_check(fv: &Value, val: &str) -> bool {
    let fv_str = frontmatter::value_to_string(fv);
    fv_str.eq_ignore_ascii_case(val)
}

fn cmp_check(fv: &Value, val: &str) -> Option<std::cmp::Ordering> {
    let fv_str = frontmatter::value_to_string(fv);
    if let (Ok(a), Ok(b)) = (fv_str.parse::<f64>(), val.parse::<f64>()) {
        return a.partial_cmp(&b);
    }
    if let (Some(a), Some(b)) = (parse_date(&fv_str), parse_date(val)) {
        return Some(a.cmp(&b));
    }
    Some(fv_str.cmp(&val.to_string()))
}

fn parse_date(s: &str) -> Option<chrono::NaiveDate> {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .ok()
        .or_else(|| {
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|dt| dt.date())
        })
}

pub fn parse_and_eval(expression: &str, fm: &Value) -> Result<bool, String> {
    let tokens = tokenize(expression)?;
    let mut parser = Parser::new(tokens);
    let expr = parser.parse_expr()?;
    Ok(evaluate(&expr, fm))
}

pub fn run(expression: &str, folder: &Path, verbose: bool, depth: usize) -> Result<(), String> {
    let files = frontmatter::collect_md_files(folder, depth);
    if files.is_empty() {
        return Err("No .md files found".to_string());
    }

    let mut found = 0;
    for file in &files {
        let doc = match frontmatter::read_file(file) {
            Ok(d) => d,
            Err(e) => {
                if verbose {
                    eprintln!("Warning: skipping {}: {e}", file.display());
                }
                continue;
            }
        };
        match parse_and_eval(expression, &doc.frontmatter) {
            Ok(true) => {
                found += 1;
                if verbose {
                    let fname = file.file_name().unwrap_or_default().to_string_lossy();
                    println!("{fname}");
                    if let Some(table) = doc.frontmatter.as_table() {
                        for (k, v) in table {
                            println!(
                                "  {}: {}",
                                k,
                                frontmatter::value_to_string(v)
                            );
                        }
                    }
                } else {
                    println!("{}", file.display());
                }
            }
            Ok(false) => {}
            Err(e) => {
                if verbose {
                    eprintln!("Query error on {}: {e}", file.display());
                }
            }
        }
    }

    if found == 0 {
        println!("No matches.");
    }
    Ok(())
}
