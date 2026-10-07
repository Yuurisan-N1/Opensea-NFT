use crate::core::error::{MintError, MintResult};
use crate::ui::logger::{ask, ask_end, lr, ly};

pub fn ask_contract_address() -> MintResult<String> {
    ly("Enter the target contract address for this mint");
    ask("> ");

    let trimmed = read_green()?;
    ask_end(&trimmed);
    if !is_contract_address(&trimmed) {
        lr("The address you entered is not a valid contract address");
        return Err(MintError::BadContract);
    }

    Ok(trimmed.to_lowercase())
}

/// Asks which of the drop's own stages to arm.
///
/// The stage set is not a fixed list: OpenSea publishes whatever labels the
/// collection uses (`whitelist stage`, `Public stage`, `FCFS`, `GTD`, ...), so
/// the caller prints the numbered stages it read from the drop and the operator
/// answers with numbers. Returns the picked numbers, 1 based, in menu order.
pub fn ask_stage_numbers(count: usize) -> MintResult<Vec<usize>> {
    ly("Enter the number of each stage to arm, separated by commas");
    ask("> ");

    let trimmed = read_green()?;
    ask_end(&trimmed);

    let mut picked: Vec<usize> = Vec::new();
    for part in trimmed.split(',') {
        let token = part.trim();
        if token.is_empty() {
            continue;
        }
        match token.parse::<usize>() {
            Ok(value) if value >= 1 && value <= count => {
                if !picked.contains(&value) {
                    picked.push(value);
                }
            }
            _ => {
                lr("One of the numbers you entered is not on the stage list");
                return Err(MintError::Input);
            }
        }
    }
    if picked.is_empty() {
        lr("You did not pick any stage to arm");
        return Err(MintError::Input);
    }

    Ok(picked)
}

pub fn is_contract_address(value: &str) -> bool {
    let body = match value.strip_prefix("0x") {
        Some(rest) => rest,
        None => return false,
    };
    body.len() == 40 && body.chars().all(|c| c.is_ascii_hexdigit())
}

/// Reads one operator line and reprints it through the logger in canon green.
///
/// The terminal's own echo is switched off first. On a console that has no
/// `stty` the typed row is erased instead, so the value can never appear twice
/// -- a raw white echo followed by the green line is the double the operator
/// saw on 2026-10-07 (`>0xc2ce...` then `> 0xc2ce...`).
fn read_green() -> MintResult<String> {
    let mut line = String::new();
    let echo = EchoGuard::off();
    let read = std::io::stdin().read_line(&mut line);
    let suppressed = echo.suppressed();
    drop(echo);
    if read.is_err() {
        return Err(MintError::Input);
    }
    if !suppressed && std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        crate::ui::logger::progress_erase();
    }

    Ok(line.trim().to_string())
}

/// Suppresses the terminal's own echo while the operator types, so the address
/// can be reprinted through the logger in canon green instead of raw white.
/// Echo is restored on every exit path, including a panic unwind.
struct EchoGuard {
    suppressed: bool,
}

impl EchoGuard {
    fn off() -> EchoGuard {
        EchoGuard {
            suppressed: set_echo(false),
        }
    }

    fn suppressed(&self) -> bool {
        self.suppressed
    }
}

impl Drop for EchoGuard {
    fn drop(&mut self) {
        set_echo(true);
    }
}

fn set_echo(on: bool) -> bool {
    // Only a real terminal has an echo setting; piping stdin must stay silent
    // instead of spitting `stty: Inappropriate ioctl for device` into the log.
    if !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        return false;
    }
    let flag = if on { "echo" } else { "-echo" };
    match std::process::Command::new("stty")
        .arg(flag)
        .stdin(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::null())
        .status()
    {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

/// Called from the Ctrl+C path so a mid-typing stop cannot leave the operator's
/// terminal with echo switched off.
pub fn restore_echo() {
    set_echo(true);
}
