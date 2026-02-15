pub fn parse_png_info(data: &[u8]) -> Result<(u32, u32, u32), String> {
    const SIG: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
    if data.len() < SIG.len() || data[..SIG.len()] != SIG {
        return Err("not a png file".to_string());
    }

    let mut pos = 8usize;
    while pos + 8 <= data.len() {
        let len = read_be_u32(data, pos)? as usize;
        let chunk_type = &data[pos + 4..pos + 8];
        pos += 8;
        if pos + len + 4 > data.len() {
            return Err("premature end".to_string());
        }
        if chunk_type == b"IHDR" {
            if len < 13 {
                return Err("premature end".to_string());
            }
            let w = read_be_u32(data, pos)?;
            let h = read_be_u32(data, pos + 4)?;
            let bit_depth = data[pos + 8];
            let color_type = data[pos + 9];
            let channels = match color_type {
                0 => 1,
                2 => 3,
                3 => 1,
                4 => 2,
                6 => 4,
                _ => return Err("unsupported png color type".to_string()),
            };
            let bits_per_pixel = if color_type == 3 {
                bit_depth as u32
            } else {
                (bit_depth as u32) * (channels as u32)
            };
            return Ok((w, h, bits_per_pixel));
        }
        pos += len + 4;
    }

    Err("premature end".to_string())
}

pub fn parse_dib_info(data: &[u8]) -> Result<(u32, u32, u32, u32), String> {
    if data.len() < 4 {
        return Err("premature end".to_string());
    }
    let header_size = read_le_u32(data, 0)? as usize;
    if header_size < 40 {
        return Err("bitmap header is too short".to_string());
    }
    if data.len() < 40 {
        return Err("premature end".to_string());
    }

    let width = read_le_i32(data, 4)?;
    let height = read_le_i32(data, 8)?;
    let planes = read_le_u16(data, 12)?;
    let bit_count = read_le_u16(data, 14)?;
    let compression = read_le_u32(data, 16)?;
    let clr_used = read_le_u32(data, 32)?;
    let clr_important = read_le_u32(data, 36)?;

    if compression != 0 {
        return Err("compressed image data not supported".to_string());
    }
    if planes != 1 {
        return Err("planes field in bitmap should be one".to_string());
    }
    if clr_important != 0 {
        return Err("clr_important field in bitmap should be zero".to_string());
    }
    if width <= 0 {
        return Err("invalid bitmap width".to_string());
    }

    let height_abs = height.unsigned_abs();
    let image_height = (height_abs / 2) as u32;
    let palette_count = if clr_used != 0 || bit_count < 24 {
        if clr_used != 0 {
            clr_used
        } else if bit_count >= 32 {
            0
        } else {
            1u32.checked_shl(bit_count as u32)
                .ok_or_else(|| "palette too large".to_string())?
        }
    } else {
        0
    };

    Ok((width as u32, image_height, bit_count as u32, palette_count))
}

fn read_be_u32(data: &[u8], offset: usize) -> Result<u32, String> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| "premature end".to_string())?;
    let bytes: [u8; 4] = data
        .get(offset..end)
        .ok_or_else(|| "premature end".to_string())?
        .try_into()
        .map_err(|_| "premature end".to_string())?;
    Ok(u32::from_be_bytes(bytes))
}

fn read_le_u32(data: &[u8], offset: usize) -> Result<u32, String> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| "premature end".to_string())?;
    let bytes: [u8; 4] = data
        .get(offset..end)
        .ok_or_else(|| "premature end".to_string())?
        .try_into()
        .map_err(|_| "premature end".to_string())?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_le_i32(data: &[u8], offset: usize) -> Result<i32, String> {
    Ok(i32::from_le_bytes(read_le_u32(data, offset)?.to_le_bytes()))
}

fn read_le_u16(data: &[u8], offset: usize) -> Result<u16, String> {
    let end = offset
        .checked_add(2)
        .ok_or_else(|| "premature end".to_string())?;
    let bytes: [u8; 2] = data
        .get(offset..end)
        .ok_or_else(|| "premature end".to_string())?
        .try_into()
        .map_err(|_| "premature end".to_string())?;
    Ok(u16::from_le_bytes(bytes))
}
