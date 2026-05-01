// SPDX-FileCopyrightText: 2026 (c) Gary "BLumia" Wang <opensource@blumia.net>
//
// SPDX-License-Identifier: MIT

use std::path::Path;

pub fn program_basename(s: &str) -> String {
    Path::new(s)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(s)
        .to_string()
}

pub fn split_long(s: &str) -> (&str, Option<&str>) {
    match s.split_once('=') {
        Some((k, v)) => (k, Some(v)),
        None => (s, None),
    }
}

pub fn take_value(
    value: Option<&str>,
    argv: &[String],
    i: &mut usize,
    opt: &str,
) -> Result<String, String> {
    if let Some(v) = value {
        return Ok(v.to_string());
    }
    let next = argv
        .get(*i + 1)
        .ok_or_else(|| format!("option '{opt}' requires an argument"))?;
    *i += 1;
    Ok(next.clone())
}

pub fn take_short_value(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    argv: &[String],
    i: &mut usize,
    opt: &str,
) -> Result<String, String> {
    if chars.peek().is_some() {
        let rest: String = chars.collect();
        return Ok(rest);
    }
    let next = argv
        .get(*i + 1)
        .ok_or_else(|| format!("option '{opt}' requires an argument"))?;
    *i += 1;
    Ok(next.clone())
}

pub fn set_command<T: Eq + Copy>(slot: &mut Option<T>, cmd: T) -> Result<(), String> {
    if let Some(existing) = slot {
        if *existing != cmd {
            return Err("multiple commands specified".to_string());
        }
        return Ok(());
    }
    *slot = Some(cmd);
    Ok(())
}

pub fn parse_i32(field: &str, value: &str) -> Result<i32, String> {
    let n: i32 = value
        .parse()
        .map_err(|_| format!("invalid {field} value: {value}"))?;
    if n < 0 {
        return Err(format!("invalid {field} value: {value}"));
    }
    Ok(n)
}
