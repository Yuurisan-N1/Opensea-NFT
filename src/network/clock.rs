use std::time::{Duration, Instant};

use crate::ui::logger::{lg, ly};

const PROBE_URLS: [&str; 4] = [
    "https://ethereum-rpc.publicnode.com",
    "https://base-rpc.publicnode.com",
    "https://arbitrum-one-rpc.publicnode.com",
    "https://polygon-bor-rpc.publicnode.com",
];

#[derive(Debug, Clone, Copy)]
pub struct ClockSync {
    offset_ms: i128,
    samples: usize,
}

impl ClockSync {
    pub fn offset_ms(&self) -> i128 {
        self.offset_ms
    }

    pub fn samples(&self) -> usize {
        self.samples
    }
}

pub async fn synchronize() -> ClockSync {
    let mut offsets: Vec<i128> = Vec::new();

    for url in PROBE_URLS {
        if let Some(offset) = probe(url).await {
            offsets.push(offset);
        }
    }

    if offsets.is_empty() {
        ly("Clock sync could not reach a time source and the local clock is used");
        return ClockSync {
            offset_ms: 0,
            samples: 0,
        };
    }

    offsets.sort_unstable();
    let middle = offsets.len() / 2;
    let offset_ms = if offsets.len() % 2 == 1 {
        offsets[middle]
    } else {
        (offsets[middle - 1] + offsets[middle]) / 2
    };

    let spread = offsets[offsets.len() - 1] - offsets[0];
    lg(&format!(
        "Clock sync completed with {} sources and {} ms spread",
        offsets.len(),
        spread
    ));

    ClockSync {
        offset_ms,
        samples: offsets.len(),
    }
}

async fn probe(url: &str) -> Option<i128> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(6))
        .build()
        .ok()?;

    let started = Instant::now();
    let response = client.head(url).send().await.ok()?;
    let round_trip = started.elapsed();
    let date = response.headers().get(reqwest::header::DATE)?;
    let server = httpdate_seconds(date.to_str().ok()?)?;

    let local = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_millis() as i128;

    let midway = round_trip.as_millis() as i128 / 2;
    Some(server * 1000 - (local - midway))
}

fn httpdate_seconds(value: &str) -> Option<i128> {
    let trimmed = value.trim();
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.len() < 5 {
        return None;
    }

    let day: i128 = parts[1].parse().ok()?;
    let month = match parts[2] {
        "Jan" => 0,
        "Feb" => 1,
        "Mar" => 2,
        "Apr" => 3,
        "May" => 4,
        "Jun" => 5,
        "Jul" => 6,
        "Aug" => 7,
        "Sep" => 8,
        "Oct" => 9,
        "Nov" => 10,
        "Dec" => 11,
        _ => return None,
    };
    let year: i128 = parts[3].parse().ok()?;
    let clock: Vec<&str> = parts[4].split(':').collect();
    if clock.len() != 3 {
        return None;
    }
    let hour: i128 = clock[0].parse().ok()?;
    let minute: i128 = clock[1].parse().ok()?;
    let second: i128 = clock[2].parse().ok()?;

    Some(days_from_civil(year, month, day) * 86_400 + hour * 3600 + minute * 60 + second)
}

fn days_from_civil(year: i128, month: i128, day: i128) -> i128 {
    let y = if month <= 1 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let m = month + 1;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

pub fn now_ms(sync: &ClockSync) -> i128 {
    let local = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i128)
        .unwrap_or(0);
    local + sync.offset_ms()
}
