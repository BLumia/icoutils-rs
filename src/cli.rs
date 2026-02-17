// SPDX-FileCopyrightText: 2026 (c) Gary "BLumia" Wang <opensource@blumia.net>
//
// SPDX-License-Identifier: MIT

use crate::types::{Action, Command, CreateInput, ParsedArgs};

pub fn parse_args(argv: &[String]) -> Result<(Action, Option<ParsedArgs>), String> {
    let mut command: Option<Command> = None;
    let mut output: Option<String> = None;

    let mut image_index: i32 = -1;
    let mut width: i32 = -1;
    let mut height: i32 = -1;
    let mut bit_depth: i32 = -1;
    let mut palette_size: i32 = -1;
    let mut hotspot_x: i32 = 0;
    let mut hotspot_y: i32 = 0;
    let mut hotspot_x_set = false;
    let mut hotspot_y_set = false;
    let mut alpha_threshold: i32 = 127;
    let mut icon_only = false;
    let mut cursor_only = false;
    let mut compat_png_bitcount = true;

    let mut files: Vec<String> = Vec::new();
    let mut create_inputs: Vec<CreateInput> = Vec::new();

    let mut i = 0usize;
    while i < argv.len() {
        let arg = &argv[i];

        if arg == "--" {
            for p in &argv[i + 1..] {
                files.push(p.clone());
                create_inputs.push(CreateInput {
                    path: p.clone(),
                    raw_png: false,
                    min_bit_depth: bit_depth,
                    hotspot_x,
                    hotspot_y,
                });
            }
            break;
        }

        if arg == "-" || !arg.starts_with('-') {
            files.push(arg.clone());
            create_inputs.push(CreateInput {
                path: arg.clone(),
                raw_png: false,
                min_bit_depth: bit_depth,
                hotspot_x,
                hotspot_y,
            });
            i += 1;
            continue;
        }

        if let Some(stripped) = arg.strip_prefix("--") {
            let (name, value) = split_long(stripped);
            match name {
                "help" => return Ok((Action::Help, None)),
                "version" => return Ok((Action::Version, None)),
                "extract" => set_command(&mut command, Command::Extract)?,
                "list" => set_command(&mut command, Command::List)?,
                "create" => set_command(&mut command, Command::Create)?,
                "icon" => icon_only = true,
                "cursor" => cursor_only = true,
                "output" => output = Some(take_value(value, argv, &mut i, "--output")?),
                "index" => {
                    image_index = parse_i32("index", take_value(value, argv, &mut i, "--index")?)?
                }
                "width" => width = parse_i32("width", take_value(value, argv, &mut i, "--width")?)?,
                "height" => {
                    height = parse_i32("height", take_value(value, argv, &mut i, "--height")?)?
                }
                "palette-size" => {
                    palette_size = parse_i32(
                        "palette-size",
                        take_value(value, argv, &mut i, "--palette-size")?,
                    )?
                }
                "bit-depth" => {
                    bit_depth =
                        parse_i32("bit-depth", take_value(value, argv, &mut i, "--bit-depth")?)?
                }
                "hotspot-x" => {
                    hotspot_x =
                        parse_i32("hotspot-x", take_value(value, argv, &mut i, "--hotspot-x")?)?;
                    hotspot_x_set = true;
                }
                "hotspot-y" => {
                    hotspot_y =
                        parse_i32("hotspot-y", take_value(value, argv, &mut i, "--hotspot-y")?)?;
                    hotspot_y_set = true;
                }
                "alpha-threshold" => {
                    alpha_threshold = parse_i32(
                        "alpha-threshold",
                        take_value(value, argv, &mut i, "--alpha-threshold")?,
                    )?
                }
                "no-compat-png-bitcount" => compat_png_bitcount = false,
                "raw" => {
                    let raw_path = take_value(value, argv, &mut i, "--raw")?;
                    files.push(raw_path.clone());
                    create_inputs.push(CreateInput {
                        path: raw_path,
                        raw_png: true,
                        min_bit_depth: bit_depth,
                        hotspot_x,
                        hotspot_y,
                    });
                }
                _ => return Err(format!("unrecognized option '--{name}'")),
            }

            i += 1;
            continue;
        }

        let mut chars = arg[1..].chars().peekable();
        while let Some(ch) = chars.next() {
            match ch {
                'x' => set_command(&mut command, Command::Extract)?,
                'l' => set_command(&mut command, Command::List)?,
                'c' => set_command(&mut command, Command::Create)?,
                'o' => output = Some(take_short_value(&mut chars, argv, &mut i, "-o")?),
                'i' => {
                    image_index =
                        parse_i32("index", take_short_value(&mut chars, argv, &mut i, "-i")?)?
                }
                'w' => {
                    width = parse_i32("width", take_short_value(&mut chars, argv, &mut i, "-w")?)?
                }
                'h' => {
                    height = parse_i32("height", take_short_value(&mut chars, argv, &mut i, "-h")?)?
                }
                'p' => {
                    palette_size = parse_i32(
                        "palette-size",
                        take_short_value(&mut chars, argv, &mut i, "-p")?,
                    )?
                }
                'b' => {
                    bit_depth = parse_i32(
                        "bit-depth",
                        take_short_value(&mut chars, argv, &mut i, "-b")?,
                    )?
                }
                'X' => {
                    hotspot_x = parse_i32(
                        "hotspot-x",
                        take_short_value(&mut chars, argv, &mut i, "-X")?,
                    )?;
                    hotspot_x_set = true;
                }
                'Y' => {
                    hotspot_y = parse_i32(
                        "hotspot-y",
                        take_short_value(&mut chars, argv, &mut i, "-Y")?,
                    )?;
                    hotspot_y_set = true;
                }
                't' => {
                    alpha_threshold = parse_i32(
                        "alpha-threshold",
                        take_short_value(&mut chars, argv, &mut i, "-t")?,
                    )?
                }
                'r' => {
                    let raw_path = take_short_value(&mut chars, argv, &mut i, "-r")?;
                    files.push(raw_path.clone());
                    create_inputs.push(CreateInput {
                        path: raw_path,
                        raw_png: true,
                        min_bit_depth: bit_depth,
                        hotspot_x,
                        hotspot_y,
                    });
                }
                _ => return Err(format!("invalid option -- '{ch}'")),
            }
        }

        i += 1;
    }

    if icon_only && cursor_only {
        return Err("only one of --icon and --cursor may be specified".to_string());
    }

    let Some(command) = command else {
        return Ok((Action::Run, None));
    };

    Ok((
        Action::Run,
        Some(ParsedArgs {
            command,
            output,
            image_index,
            width,
            height,
            bit_depth,
            palette_size,
            hotspot_x,
            hotspot_y,
            hotspot_x_set,
            hotspot_y_set,
            alpha_threshold,
            icon_only,
            cursor_only,
            compat_png_bitcount,
            files,
            create_inputs,
        }),
    ))
}

pub fn print_help(program_name: &str) {
    println!("Usage: {program_name} [OPTION]... [FILE]...");
    println!("Convert and create Win32 icon (.ico) and cursor (.cur) files.");
    println!();
    println!("Commands:");
    println!("  -x, --extract                extract images from files");
    println!("  -l, --list                   print a list of images in files");
    println!("  -c, --create                 create an icon file from specified files");
    println!("      --help                   display this help and exit");
    println!("      --version                output version information and exit");
    println!();
    println!("Options:");
    println!("  -i, --index=NUMBER           match index of image (first is 1)");
    println!("  -w, --width=PIXELS           match width of image");
    println!("  -h, --height=PIXELS          match height of image");
    println!("  -p, --palette-size=COUNT     match number of colors in palette (or 0)");
    println!("  -b, --bit-depth=COUNT        match or set number of bits per pixel");
    println!("  -X, --hotspot-x=COORD        match or set cursor hotspot x-coordinate");
    println!("  -Y, --hotspot-y=COORD        match or set cursor hotspot y-coordinate");
    println!(
        "  -t, --alpha-threshold=LEVEL  highest level in alpha channel indicating\n\
                               transparent image portions (default is 127)"
    );
    println!("  -r, --raw=FILENAME           store input file as raw PNG (\"Vista icons\")");
    println!("      --no-compat-png-bitcount write PNG entry bit count from IHDR");
    println!("      --icon                   match icons only");
    println!("      --cursor                 match cursors only");
    println!("  -o, --output=PATH            where to place extracted files");
    println!();
}

pub fn print_version(program_name: &str) {
    println!("{program_name} 0.1.0");
}

fn set_command(slot: &mut Option<Command>, cmd: Command) -> Result<(), String> {
    if let Some(existing) = slot {
        if *existing != cmd {
            return Err("multiple commands specified".to_string());
        }
        return Ok(());
    }
    *slot = Some(cmd);
    Ok(())
}

fn split_long(s: &str) -> (&str, Option<&str>) {
    match s.split_once('=') {
        Some((k, v)) => (k, Some(v)),
        None => (s, None),
    }
}

fn take_value(
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

fn take_short_value(
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

fn parse_i32(field: &str, value: String) -> Result<i32, String> {
    let n: i32 = value
        .parse()
        .map_err(|_| format!("invalid {field} value: {value}"))?;
    if n < 0 {
        return Err(format!("invalid {field} value: {value}"));
    }
    Ok(n)
}
