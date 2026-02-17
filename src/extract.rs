// SPDX-FileCopyrightText: 2026 (c) Gary "BLumia" Wang <opensource@blumia.net>
//
// SPDX-License-Identifier: MIT

use crate::{
    input::read_input,
    parse::{parse_dib_info, parse_png_info},
    types::{EntryMeta, ParsedArgs},
};
use std::io::{self, Write};

pub fn run_extract(args: &ParsedArgs) -> i32 {
    if args.files.is_empty() {
        eprintln!("missing arguments");
        return 1;
    }

    for file in &args.files {
        let display_name = if file == "-" {
            "(standard in)"
        } else {
            file.as_str()
        };

        let bytes = match read_input(file) {
            Ok(bytes) => bytes,
            Err(_) => {
                eprintln!("{file}: cannot open file");
                continue;
            }
        };

        let matched = match extract_from_bytes(&bytes, display_name, args) {
            Ok(matched) => matched,
            Err(msg) => {
                eprintln!("{display_name}: {msg}");
                return 1;
            }
        };

        if matched == 0 {
            eprintln!("{display_name}: no images matched");
        }
    }

    0
}

fn extract_from_bytes(bytes: &[u8], inname: &str, args: &ParsedArgs) -> Result<usize, String> {
    let cursor = std::io::Cursor::new(bytes);
    let icon_dir =
        ico::IconDir::read(cursor).map_err(|_| "not an icon or cursor file".to_string())?;

    let mut matched = 0usize;
    for (i, entry) in icon_dir.entries().iter().enumerate() {
        let index = (i + 1) as i32;
        let meta = entry_to_meta(index, entry)?;
        if !matches_filters(args, &meta) {
            continue;
        }
        matched += 1;

        let (mut out, outname) = open_extract_output(inname, &args.output, &meta)?;
        if entry.is_png() {
            out.write_all(entry.data())
                .map_err(|_| format!("{outname}: cannot write to file"))?;
        } else {
            let image = entry
                .decode()
                .map_err(|_| "failed to decode bitmap entry".to_string())?;
            image
                .write_png(&mut out)
                .map_err(|_| format!("{outname}: cannot write to file"))?;
        }
        out.flush().ok();
    }

    Ok(matched)
}

fn entry_to_meta(index: i32, entry: &ico::IconDirEntry) -> Result<EntryMeta, String> {
    let is_icon = entry.resource_type() == ico::ResourceType::Icon;
    let (width, height, bit_depth, palette_size) = if entry.is_png() {
        let (w, h, bpp) = parse_png_info(entry.data())?;
        (w as i32, h as i32, bpp as i32, 0i32)
    } else {
        let (w, h, bpp, pal) = parse_dib_info(entry.data())?;
        (w as i32, h as i32, bpp as i32, pal as i32)
    };

    let (hotspot_x, hotspot_y) = match entry.cursor_hotspot() {
        Some((x, y)) => (x as i32, y as i32),
        None => (0, 0),
    };

    Ok(EntryMeta {
        index,
        width,
        height,
        bit_depth,
        palette_size,
        is_icon,
        hotspot_x,
        hotspot_y,
    })
}

fn matches_filters(args: &ParsedArgs, meta: &EntryMeta) -> bool {
    if args.image_index != -1 && meta.index != args.image_index {
        return false;
    }
    if args.width != -1 && meta.width != args.width {
        return false;
    }
    if args.height != -1 && meta.height != args.height {
        return false;
    }
    if args.bit_depth != -1 && meta.bit_depth != args.bit_depth {
        return false;
    }
    if args.palette_size != -1 && meta.palette_size != args.palette_size {
        return false;
    }
    if (args.icon_only && !meta.is_icon) || (args.cursor_only && meta.is_icon) {
        return false;
    }

    let (hx, hy) = if meta.is_icon {
        (0, 0)
    } else {
        (meta.hotspot_x, meta.hotspot_y)
    };

    if args.hotspot_x_set && hx != args.hotspot_x {
        return false;
    }
    if args.hotspot_y_set && hy != args.hotspot_y {
        return false;
    }

    true
}

fn open_extract_output(
    inname: &str,
    output: &Option<String>,
    meta: &EntryMeta,
) -> Result<(Box<dyn Write>, String), String> {
    let Some(output) = output.as_deref() else {
        let path = gen_extract_name(inname, None, meta);
        let f = std::fs::File::create(&path).map_err(|_| format!("{path}: cannot create file"))?;
        return Ok((Box::new(f), path));
    };

    let output_is_dir = std::fs::metadata(output)
        .map(|m| m.is_dir())
        .unwrap_or(false);
    if output_is_dir {
        let path = gen_extract_name(inname, Some(output), meta);
        let f = std::fs::File::create(&path).map_err(|_| format!("{path}: cannot create file"))?;
        return Ok((Box::new(f), path));
    }

    if output == "-" {
        return Ok((Box::new(io::stdout().lock()), "(standard out)".to_string()));
    }

    let f = std::fs::File::create(output).map_err(|_| format!("{output}: cannot create file"))?;
    Ok((Box::new(f), output.to_string()))
}

fn gen_extract_name(inname: &str, output_dir: Option<&str>, meta: &EntryMeta) -> String {
    let mut base = inname;
    if let Some(pos) = inname.rfind(['/', '\\']) {
        base = &inname[pos + 1..];
    }

    let stem = strip_ico_cur_ext(base);
    let filename = format!(
        "{stem}_{}_{}x{}x{}.png",
        meta.index, meta.width, meta.height, meta.bit_depth
    );

    match output_dir {
        Some(dir) => std::path::Path::new(dir)
            .join(filename)
            .to_string_lossy()
            .to_string(),
        None => filename,
    }
}

fn strip_ico_cur_ext(name: &str) -> &str {
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".ico") || lower.ends_with(".cur") {
        &name[..name.len() - 4]
    } else {
        name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rgba_png_1x1() -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut buf, 1, 1);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(&[1, 2, 3, 4]).unwrap();
        }
        buf
    }

    fn build_ico_with_png(png_bytes: &[u8], width: u8, height: u8) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());

        out.push(width);
        out.push(height);
        out.push(0);
        out.push(0);
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&32u16.to_le_bytes());
        out.extend_from_slice(&(png_bytes.len() as u32).to_le_bytes());
        out.extend_from_slice(&(6u32 + 16u32).to_le_bytes());

        out.extend_from_slice(png_bytes);
        out
    }

    #[test]
    fn extract_raw_png_entry_writes_original_png_bytes() {
        let png_bytes = make_rgba_png_1x1();
        let ico_bytes = build_ico_with_png(&png_bytes, 1, 1);

        let cursor = std::io::Cursor::new(&ico_bytes);
        let icon_dir = ico::IconDir::read(cursor).unwrap();
        assert!(icon_dir.entries()[0].is_png());

        let meta = entry_to_meta(1, &icon_dir.entries()[0]).unwrap();
        let name = gen_extract_name("a/b/c.ico", None, &meta);
        assert_eq!(name, "c_1_1x1x32.png");
        let name2 = gen_extract_name(r"a\b\c.CUR", Some("outdir"), &meta);
        assert!(name2.ends_with(r"outdir\c_1_1x1x32.png"));

        let data = icon_dir.entries()[0].data();
        assert_eq!(data, png_bytes.as_slice());
    }
}
