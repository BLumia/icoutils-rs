// SPDX-FileCopyrightText: 2026 (c) Gary "BLumia" Wang <opensource@blumia.net>
//
// SPDX-License-Identifier: MIT

use crate::{
    input::read_input,
    parse::{parse_dib_info, parse_png_info},
    types::{CreateInput, ParsedArgs},
};
use std::io::{self, IsTerminal, Write};

pub fn run_create(args: &ParsedArgs) -> i32 {
    if args.create_inputs.is_empty() {
        eprintln!("missing file argument");
        return 1;
    }

    let out = match &args.output {
        Some(p) => p.as_str(),
        None => "",
    };

    let write_to_stdout = args.output.is_none() || out == "-";
    if write_to_stdout && io::stdout().is_terminal() {
        eprintln!("refusing to write binary data to terminal (use -o FILE or -o -)");
        return 1;
    }

    let is_cursor = args.cursor_only && !args.icon_only;
    let container_type: u16 = if is_cursor { 2 } else { 1 };

    let mut images: Vec<EncodedImage> = Vec::with_capacity(args.create_inputs.len());
    for input in &args.create_inputs {
        let bytes = match read_input(&input.path) {
            Ok(b) => b,
            Err(_) => {
                eprintln!("{}: cannot open file", input.path);
                return 1;
            }
        };

        let encoded = match encode_one(input, &bytes) {
            Ok(v) => v,
            Err(msg) => {
                eprintln!("{}: {msg}", input.path);
                return 1;
            }
        };
        images.push(encoded);
    }

    let out_bytes = build_ico(container_type, is_cursor, &images, args.compat_png_bitcount);

    if write_to_stdout {
        let mut stdout = io::stdout().lock();
        if stdout.write_all(&out_bytes).is_err() {
            eprintln!("cannot write output");
            return 1;
        }
        return 0;
    }

    let out_path = out;
    if std::fs::metadata(out_path)
        .map(|m| m.is_dir())
        .unwrap_or(false)
    {
        eprintln!("{out_path}: is a directory");
        return 1;
    }

    if std::fs::write(out_path, out_bytes).is_err() {
        eprintln!("{out_path}: cannot write file");
        return 1;
    }

    0
}

struct EncodedImage {
    width: u32,
    height: u32,
    bit_depth: u32,
    hotspot_x: u16,
    hotspot_y: u16,
    data: Vec<u8>,
}

fn encode_one(input: &CreateInput, bytes: &[u8]) -> Result<EncodedImage, String> {
    if input.raw_png {
        let (w, h, bpp) = parse_png_info(bytes)?;
        return Ok(EncodedImage {
            width: w,
            height: h,
            bit_depth: bpp,
            hotspot_x: clamp_u16(input.hotspot_x),
            hotspot_y: clamp_u16(input.hotspot_y),
            data: bytes.to_vec(),
        });
    }

    let (w, h, rgba) = decode_png_rgba(bytes)?;
    let image = ico::IconImage::from_rgba_data(w, h, rgba);
    let entry =
        ico::IconDirEntry::encode(&image).map_err(|_| "failed to encode image".to_string())?;
    let data = entry.data().to_vec();

    let (width, height, bit_depth) = if is_png_bytes(&data) {
        let (w, h, bpp) = parse_png_info(&data)?;
        (w, h, bpp)
    } else {
        let (w, h, bpp, _) = parse_dib_info(&data)?;
        (w, h, bpp)
    };

    Ok(EncodedImage {
        width,
        height,
        bit_depth,
        hotspot_x: clamp_u16(input.hotspot_x),
        hotspot_y: clamp_u16(input.hotspot_y),
        data,
    })
}

fn build_ico(
    container_type: u16,
    is_cursor: bool,
    images: &[EncodedImage],
    compat_png_bitcount: bool,
) -> Vec<u8> {
    let count = images.len() as u16;
    let header_size = 6usize;
    let entry_size = 16usize;
    let data_offset_base = header_size + entry_size * (images.len());

    let mut out =
        Vec::with_capacity(data_offset_base + images.iter().map(|i| i.data.len()).sum::<usize>());

    write_u16_le(&mut out, 0);
    write_u16_le(&mut out, container_type);
    write_u16_le(&mut out, count);

    let mut offsets: Vec<u32> = Vec::with_capacity(images.len());
    let mut cur = data_offset_base as u32;
    for img in images {
        offsets.push(cur);
        cur = cur.wrapping_add(img.data.len() as u32);
    }

    for (img, &offset) in images.iter().zip(offsets.iter()) {
        let width_byte = to_dim_byte(img.width);
        let height_byte = to_dim_byte(img.height);
        out.push(width_byte);
        out.push(height_byte);

        let color_count: u8 = 0;
        out.push(color_count);
        out.push(0);

        if is_cursor {
            write_u16_le(&mut out, img.hotspot_x);
            write_u16_le(&mut out, img.hotspot_y);
        } else {
            write_u16_le(&mut out, 1);
            let bit_count = if compat_png_bitcount && is_png_bytes(&img.data) {
                // icoutils writes 32 here for PNG-compressed entries; using the PNG IHDR derived
                // bpp (e.g. 64 for RGBA16) causes avoidable binary diffs.
                //
                // This can be disabled with --no-compat-png-bitcount.
                32i32
            } else {
                img.bit_depth as i32
            };
            write_u16_le(&mut out, clamp_u16(bit_count));
        }

        write_u32_le(&mut out, img.data.len() as u32);
        write_u32_le(&mut out, offset);
    }

    for img in images {
        out.extend_from_slice(&img.data);
    }

    out
}

fn to_dim_byte(dim: u32) -> u8 {
    if dim >= 256 { 0 } else { dim as u8 }
}

fn is_png_bytes(data: &[u8]) -> bool {
    const SIG: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
    data.len() >= SIG.len() && data[..SIG.len()] == SIG
}

fn decode_png_rgba(data: &[u8]) -> Result<(u32, u32, Vec<u8>), String> {
    let decoder = png::Decoder::new(std::io::Cursor::new(data));
    let mut reader = decoder
        .read_info()
        .map_err(|_| "not a png file".to_string())?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|_| "failed to decode png".to_string())?;
    let bytes = &buf[..info.buffer_size()];

    let rgba = match info.color_type {
        png::ColorType::Rgba => bytes.to_vec(),
        png::ColorType::Rgb => rgb_to_rgba(bytes),
        png::ColorType::Grayscale => gray_to_rgba(bytes),
        png::ColorType::GrayscaleAlpha => gray_alpha_to_rgba(bytes),
        png::ColorType::Indexed => indexed_to_rgba(&mut reader, bytes)?,
    };

    Ok((info.width, info.height, rgba))
}

fn rgb_to_rgba(rgb: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(rgb.len() / 3 * 4);
    for chunk in rgb.chunks_exact(3) {
        out.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
    }
    out
}

fn gray_to_rgba(gray: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(gray.len() * 4);
    for &g in gray {
        out.extend_from_slice(&[g, g, g, 255]);
    }
    out
}

fn gray_alpha_to_rgba(ga: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(ga.len() / 2 * 4);
    for chunk in ga.chunks_exact(2) {
        let g = chunk[0];
        let a = chunk[1];
        out.extend_from_slice(&[g, g, g, a]);
    }
    out
}

fn indexed_to_rgba(
    reader: &mut png::Reader<std::io::Cursor<&[u8]>>,
    idx: &[u8],
) -> Result<Vec<u8>, String> {
    let palette = reader
        .info()
        .palette
        .as_ref()
        .ok_or_else(|| "png palette missing".to_string())?;
    let trns = reader.info().trns.as_deref();

    let mut out = Vec::with_capacity(idx.len() * 4);
    for &i in idx {
        let base = (i as usize) * 3;
        if base + 2 >= palette.len() {
            return Err("png palette index out of range".to_string());
        }
        let r = palette[base];
        let g = palette[base + 1];
        let b = palette[base + 2];
        let a = trns.and_then(|t| t.get(i as usize).copied()).unwrap_or(255);
        out.extend_from_slice(&[r, g, b, a]);
    }
    Ok(out)
}

fn clamp_u16(v: i32) -> u16 {
    if v <= 0 {
        0
    } else if v >= u16::MAX as i32 {
        u16::MAX
    } else {
        v as u16
    }
}

fn write_u16_le(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn write_u32_le(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
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
            writer.write_image_data(&[0, 0, 0, 0]).unwrap();
        }
        buf
    }

    fn make_rgba16_png_1x1() -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut buf, 1, 1);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Sixteen);
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(&[0, 0, 0, 0, 0, 0, 0, 0]).unwrap();
        }
        buf
    }

    #[test]
    fn create_raw_png_cursor_writes_hotspot_fields() {
        let bytes = make_rgba_png_1x1();
        let input = CreateInput {
            path: "mem".to_string(),
            raw_png: true,
            min_bit_depth: -1,
            hotspot_x: 7,
            hotspot_y: 9,
        };
        let img = encode_one(&input, &bytes).unwrap();
        let ico = build_ico(2, true, &[img], true);

        assert_eq!(&ico[0..2], &[0, 0]);
        assert_eq!(u16::from_le_bytes([ico[2], ico[3]]), 2);
        assert_eq!(u16::from_le_bytes([ico[4], ico[5]]), 1);
        assert_eq!(ico[6], 1);
        assert_eq!(ico[7], 1);
        assert_eq!(u16::from_le_bytes([ico[10], ico[11]]), 7);
        assert_eq!(u16::from_le_bytes([ico[12], ico[13]]), 9);
    }

    #[test]
    fn create_raw_png_rgba16_normalizes_icon_bit_count_to_32() {
        let bytes = make_rgba16_png_1x1();
        let input = CreateInput {
            path: "mem".to_string(),
            raw_png: true,
            min_bit_depth: -1,
            hotspot_x: 0,
            hotspot_y: 0,
        };
        let img = encode_one(&input, &bytes).unwrap();
        let ico = build_ico(1, false, &[img], true);

        assert_eq!(u16::from_le_bytes([ico[2], ico[3]]), 1);
        assert_eq!(u16::from_le_bytes([ico[4], ico[5]]), 1);
        assert_eq!(u16::from_le_bytes([ico[10], ico[11]]), 1);
        assert_eq!(u16::from_le_bytes([ico[12], ico[13]]), 32);
    }

    #[test]
    fn create_raw_png_rgba16_can_write_ihdr_bit_count() {
        let bytes = make_rgba16_png_1x1();
        let input = CreateInput {
            path: "mem".to_string(),
            raw_png: true,
            min_bit_depth: -1,
            hotspot_x: 0,
            hotspot_y: 0,
        };
        let img = encode_one(&input, &bytes).unwrap();
        let ico = build_ico(1, false, &[img], false);

        assert_eq!(u16::from_le_bytes([ico[12], ico[13]]), 64);
    }
}
