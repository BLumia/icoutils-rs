#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Extract,
    List,
    Create,
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

    pub files: Vec<String>,
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
