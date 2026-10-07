use ethers::types::U256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageState {
    Upcoming,
    Active,
    Ended,
    Disabled,
}

impl StageState {
    pub fn label(&self) -> &'static str {
        match self {
            StageState::Upcoming => "upcoming",
            StageState::Active => "active",
            StageState::Ended => "ended",
            StageState::Disabled => "disabled",
        }
    }
}

#[derive(Debug, Clone)]
pub struct StageWindow {
    pub label: String,
    pub start: u64,
    pub end: u64,
    pub price: U256,
    pub limit_per_wallet: u64,
    pub signed: bool,
    pub state: StageState,
}

pub fn classify(start: u64, end: u64, now: i128) -> StageState {
    if start == 0 {
        return StageState::Disabled;
    }
    if now < start as i128 {
        return StageState::Upcoming;
    }
    if end == 0 || now <= end as i128 {
        return StageState::Active;
    }
    StageState::Ended
}

pub fn build_window(
    label: &str,
    start: u64,
    end: u64,
    price: U256,
    limit_per_wallet: u64,
    signed: bool,
    now_ms: i128,
) -> StageWindow {
    StageWindow {
        label: label.to_string(),
        start,
        end,
        price,
        limit_per_wallet,
        signed,
        state: classify(start, end, now_ms / 1000),
    }
}

pub fn ms_until(start: u64, now_ms: i128) -> i128 {
    (start as i128) * 1000 - now_ms
}

pub fn menu(windows: &[StageWindow]) -> Vec<StageWindow> {
    let mut rows: Vec<StageWindow> = Vec::new();
    for window in windows {
        if window.state == StageState::Disabled || window.state == StageState::Ended {
            continue;
        }
        if rows.iter().any(|row| row.label == window.label) {
            continue;
        }
        rows.push(window.clone());
    }
    rows.sort_by_key(|row| row.start);
    rows
}

pub fn pick_next(windows: &[StageWindow], now_ms: i128) -> Option<&StageWindow> {
    windows
        .iter()
        .filter(|window| window.state == StageState::Upcoming || window.state == StageState::Active)
        .filter(|window| window.end == 0 || ms_until(window.end, now_ms) > 0)
        .min_by_key(|window| ms_until(window.start, now_ms).max(0))
}
