// SPDX-FileCopyrightText: 2026 (c) Gary "BLumia" Wang <opensource@blumia.net>
//
// SPDX-License-Identifier: MIT

use crate::{
    input::read_input,
    parse::{parse_dib_info, parse_png_info},
    types::{EntryMeta, ParsedArgs},
};

pub fn run_list(args: &ParsedArgs) -> i32 {
    if args.files.is_empty() {
        eprintln!("missing file argument");
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

        let (matched, lines) = match list_from_bytes(&bytes, args) {
            Ok(result) => result,
            Err(msg) => {
                eprintln!("{display_name}: {msg}");
                return 1;
            }
        };

        if matched == 0 {
            return 1;
        }

        for line in lines {
            println!("{line}");
        }
    }

    0
}

pub fn list_from_bytes(bytes: &[u8], args: &ParsedArgs) -> Result<(usize, Vec<String>), String> {
    let cursor = std::io::Cursor::new(bytes);
    let icon_dir =
        ico::IconDir::read(cursor).map_err(|_| "not an icon or cursor file".to_string())?;

    let mut matched = 0usize;
    let mut lines = Vec::new();
    for (i, entry) in icon_dir.entries().iter().enumerate() {
        let index = (i + 1) as i32;
        let meta = entry_to_meta(index, entry)?;
        if matches_filters(args, &meta) {
            matched += 1;
            lines.push(format_list_line(&meta));
        }
    }

    Ok((matched, lines))
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

fn format_list_line(meta: &EntryMeta) -> String {
    let kind = if meta.is_icon { "icon" } else { "cursor" };
    let mut line = format!(
        "--{} --index={} --width={} --height={} --bit-depth={} --palette-size={}",
        kind, meta.index, meta.width, meta.height, meta.bit_depth, meta.palette_size
    );
    if !meta.is_icon {
        line.push_str(&format!(
            " --hotspot-x={} --hotspot-y={}",
            meta.hotspot_x, meta.hotspot_y
        ));
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Command;

    #[test]
    fn list_single_icon_entry() {
        let mut dir = ico::IconDir::new(ico::ResourceType::Icon);
        let image = ico::IconImage::from_rgba_data(16, 16, vec![0u8; 4 * 16 * 16]);
        dir.add_entry(ico::IconDirEntry::encode(&image).unwrap());
        let mut bytes = Vec::new();
        dir.write(&mut bytes).unwrap();

        let args = ParsedArgs {
            command: Command::List,
            output: None,
            image_index: -1,
            width: -1,
            height: -1,
            bit_depth: -1,
            palette_size: -1,
            hotspot_x: 0,
            hotspot_y: 0,
            hotspot_x_set: false,
            hotspot_y_set: false,
            alpha_threshold: 127,
            icon_only: false,
            cursor_only: false,
            compat_png_bitcount: true,
            files: vec![],
            create_inputs: vec![],
        };

        let (matched, lines) = list_from_bytes(&bytes, &args).unwrap();
        assert_eq!(matched, 1);
        assert!(lines[0].starts_with("--icon --index=1 --width=16 --height=16 "));
    }
}
