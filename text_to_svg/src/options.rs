pub struct Options {
    pub x: u16,
    pub y: u16,
    pub fontsize: u16,
    pub kerning: bool,
    pub letter_spacing: Option<u16>,
    pub tracking: Option<u16>,
    pub anchor: TextAnchor,
}

pub struct TextAnchor {
    pub horizontal: TextAnchorHorizontal,
    pub vertical: TextAnchorVertical,
}

pub enum TextAnchorHorizontal {
    Left,
    Center,
    Right,
}

pub enum TextAnchorVertical {
    Baseline,
    Top,
    Middle,
    Bottom,
}
