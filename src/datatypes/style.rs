use datatypes::Color;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Style {
    Underline(u8),
    Bold(bool),
    Italic(bool),
    Blink(bool),
    InvertColors(bool),
    Strikethrough(bool),
    Opacity(u8),
    FgColor(Color),
    FgColorCfg(Option<u8>),
    BgColor(Color),
    BgColorCfg(Option<u8>),
}
