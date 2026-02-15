fn main() {
    let mut it = std::env::args();
    let program_path = it.next().unwrap_or_else(|| "icotool".to_string());
    let argv: Vec<String> = it.collect();
    std::process::exit(icotool::run_from_args(&program_path, &argv));
}
