use std::path::Path;

pub fn program_basename(s: &str) -> String {
    Path::new(s)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(s)
        .to_string()
}
