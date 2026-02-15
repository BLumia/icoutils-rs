// SPDX-FileCopyrightText: 2026 (c) Gary "BLumia" Wang <opensource@blumia.net>
//
// SPDX-License-Identifier: MIT

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Extract,
    List,
    Create,
}

#[derive(Clone, Debug)]
pub struct CreateInput {
    pub path: String,
    pub raw_png: bool,
    pub min_bit_depth: i32,
    pub hotspot_x: i32,
    pub hotspot_y: i32,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct ParsedArgs {
    pub command: Command,
    pub output: Option<String>,

    pub image_index: i32,
    pub width: i32,
    pub height: i32,
    pub bit_depth: i32,
    pub palette_size: i32,
    pub hotspot_x: i32,
    pub hotspot_y: i32,
    pub hotspot_x_set: bool,
    pub hotspot_y_set: bool,
    pub alpha_threshold: i32,
    pub icon_only: bool,
    pub cursor_only: bool,
    pub compat_png_bitcount: bool,

    pub files: Vec<String>,
    pub create_inputs: Vec<CreateInput>,
}

#[derive(Clone, Debug)]
pub struct EntryMeta {
    pub index: i32,
    pub width: i32,
    pub height: i32,
    pub bit_depth: i32,
    pub palette_size: i32,
    pub is_icon: bool,
    pub hotspot_x: i32,
    pub hotspot_y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Run,
    Help,
    Version,
}
