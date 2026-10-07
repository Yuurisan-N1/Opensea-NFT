const G: &str = "\x1b[1;32m";
const Y: &str = "\x1b[1;33m";
const R: &str = "\x1b[1;31m";
const Z: &str = "\x1b[0m";

fn line(color: &str, msg: &str) {
    println!("{}{}{}", color, clean_text(msg), Z);
}

pub fn lg(msg: &str) {
    line(G, msg);
}

pub fn ly(msg: &str) {
    line(Y, msg);
}

pub fn lr(msg: &str) {
    line(R, msg);
}

pub fn lb(msg: &str) {
    line(G, msg);
}

pub fn lm(msg: &str) {
    line(G, msg);
}

pub fn ld(msg: &str) {
    line(G, msg);
}

pub fn ask(msg: &str) {
    print!("{}{}{}", Y, clean_text(msg), Z);
    let _ = std::io::Write::flush(&mut std::io::stdout());
}

pub fn ask_end(value: &str) {
    lg(&format!("> {}", value));
}

/// Closes a live in-place row (countdown / progress) so the next writer starts
/// on its own row — the canon Ctrl+C stop shape.
pub fn close_row() {
    line(R, "");
}

pub fn sanitize(input: &str) -> String {
    let mut out: String = input
        .chars()
        .map(|c| match c {
            '[' | ']' | '#' | '-' | '|' => ' ',
            other => other,
        })
        .collect();
    for key in [
        "accessToken",
        "access_token",
        "refreshToken",
        "initData",
        "token",
        "hash=",
        "Authorization",
    ] {
        out = redact_after(&out, key);
    }
    clean_text(&out)
}

fn redact_after(input: &str, key: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(pos) = rest.find(key) {
        out.push_str(&rest[..pos]);
        out.push_str(key);
        let tail = &rest[pos + key.len()..];
        let mut idx = 0;
        let bytes = tail.as_bytes();
        while idx < bytes.len()
            && (bytes[idx] == b'"'
                || bytes[idx] == b':'
                || bytes[idx] == b'='
                || bytes[idx] == b' ')
        {
            out.push(bytes[idx] as char);
            idx += 1;
        }
        while idx < bytes.len() {
            let c = bytes[idx] as char;
            if c == '"' || c == '&' || c == ' ' || c == ',' || c == '}' || c == '\n' {
                break;
            }
            idx += 1;
        }
        out.push_str("***");
        rest = &tail[idx..];
    }
    out.push_str(rest);
    out
}

pub fn clean_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut sp = false;
    for c in input.chars() {
        if c.is_control() || c == '\u{feff}' {
            if c == '\n' || c == '\r' || c == '\t' {
                sp = true;
            }
            continue;
        }
        if c.is_whitespace() {
            sp = true;
            continue;
        }
        if sp && !out.is_empty() {
            out.push(' ');
        }
        sp = false;
        out.push(c);
    }
    out
}

pub fn short(input: &str, max: usize) -> String {
    let s = clean_text(input);
    if s.chars().count() <= max {
        return s;
    }
    let cut: String = s.chars().take(max + 1).collect();
    match cut.rfind(' ') {
        Some(i) if i > 0 => cut[..i].trim_end().to_string(),
        _ => s.chars().take(max).collect(),
    }
}

pub fn progress_line(label: &str, left: u64) {
    print!(
        "\r{}{} {}{}",
        Y,
        clean_text(label),
        left_text(left),
        Z
    );
    let _ = std::io::Write::flush(&mut std::io::stdout());
}

pub fn progress_end() {
    println!();
}

/// Erases the row the operator just typed on, so a value read without terminal
/// echo suppression (Windows has no `stty`) is printed once, in green, instead
/// of a raw white echo plus the logger line.
pub fn progress_erase() {
    const CLR: &str = "\x1b[1A\r\x1b[2K";
    print!("{}", CLR);
    let _ = std::io::Write::flush(&mut std::io::stdout());
}

/// Stage start and end times as the operator's own clock shows them.
pub fn when(secs: u64) -> String {
    use chrono::TimeZone;
    match chrono::Local.timestamp_opt(secs as i64, 0).single() {
        Some(stamp) => stamp.format("%Y-%m-%d %H:%M").to_string(),
        None => "unknown time".to_string(),
    }
}

pub async fn countdown(seconds: u64, label: &str) {
    let text = clean_text(label);
    let mut left = seconds;
    while left > 0 {
        print!(
            "\r{}{} {}{}",
            Y,
            text,
            left_text(left),
            Z
        );
        let _ = std::io::Write::flush(&mut std::io::stdout());
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        left -= 1;
    }
    println!();
}

/// `DD:HH:MM:SS` once the wait passes a day, otherwise `HH:MM:SS` — hours never
/// run past 24.
fn left_text(left: u64) -> String {
    let days = left / 86_400;
    let hours = (left % 86_400) / 3600;
    let minutes = (left % 3600) / 60;
    let seconds = left % 60;
    if days > 0 {
        format!("{:02}:{:02}:{:02}:{:02}", days, hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

pub fn clock(secs: i64) -> String {
    let s = secs.max(0);
    let d = s / 86400;
    let h = (s % 86400) / 3600;
    let m = (s % 3600) / 60;
    let sec = s % 60;
    if d > 0 {
        format!("{}d {:02}:{:02}:{:02}", d, h, m, sec)
    } else {
        format!("{:02}:{:02}:{:02}", h, m, sec)
    }
}

pub fn num(n: f64) -> String {
    let v = n.abs();
    if v >= 1_000_000_000.0 {
        format!("{:.2}B", n / 1_000_000_000.0)
    } else if v >= 1_000_000.0 {
        format!("{:.2}M", n / 1_000_000.0)
    } else if v >= 1_000.0 {
        format!("{:.2}K", n / 1_000.0)
    } else {
        format!("{}", n.round() as i64)
    }
}
