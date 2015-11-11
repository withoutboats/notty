use datatypes::args::*;

pub trait Argument: Copy + Sized {

    fn from_nums<T>(T, Option<Self>) -> Option<Self> where T: Iterator<Item=u32>;
    fn encode(&self) -> String;

    fn decode(args: Option<&str>, default: Option<Self>) -> Option<Self> {
        let iter = args.iter().flat_map(|s| s.split('.')).flat_map(|s| u32::from_str_radix(s, 16));
        Self::from_nums(iter, default)
    }

}

impl Argument for Area {

    fn from_nums<T>(mut args: T, default: Option<Area>) -> Option<Area>
    where T: Iterator<Item=u32> {
        match args.next() {
            Some(1) => Some(CursorCell),
            Some(2) => Some(CursorRow),
            Some(3) => Some(CursorColumn),
            Some(4) => Movement::from_nums(args, None).map(CursorTo),
            Some(5) => Coords::from_nums(args, None).map(CursorBound),
            Some(6) => Region::from_nums(args, None).map(Bound).or(Some(WholeScreen)),
            Some(7) => match (args.next(), args.next()) {
                (Some(top), Some(bottom))   => Some(Rows(top, bottom)),
                _                           => Some(WholeScreen),
            },
            Some(8) => match (args.next(), args.next()) {
                (Some(top), Some(bottom))   => Some(Columns(top, bottom)),
                _                           => Some(WholeScreen),
            },
            Some(9) => bool::from_nums(args, Some(true)).map(BelowCursor),
            _       => default,
        }
    }

    fn encode(&self) -> String {
        match *self {
            CursorCell              => String::from("1"),
            CursorRow               => String::from("2"),
            CursorColumn            => String::from("3"),
            CursorTo(mov)           => format!("4.{}", mov.encode()),
            CursorBound(coords)     => format!("5.{}", coords.encode()),
            WholeScreen             => format!("6"),
            Bound(region)           => format!("6.{}", region.encode()),
            Rows(top, bottom)       => format!("7.{:x}.{:x}", top, bottom),
            Columns(left, right)    => format!("8.{:x}.{:x}", left, right),
            BelowCursor(b)          => format!("9.{}", b.encode()),
        }
    }

}

impl Argument for bool {

    fn from_nums<T>(mut args: T, default: Option<bool>) -> Option<bool>
    where T: Iterator<Item=u32> {
        args.next().map_or(default, |n| match n {
            0   => Some(false),
            1   => Some(true),
            _   => default,
        })
    }

    fn encode(&self) -> String {
        if *self { String::from("1") } else { String::from("0") }
    }

}

impl Argument for Color {

    fn from_nums<T>(mut args: T, default: Option<Color>) -> Option<Color>
    where T: Iterator<Item=u32> {
        match (args.next(), args.next(), args.next()) {
            (Some(r), Some(g), Some(b)) => Some(Color(r as u8, g as u8, b as u8)),
            _                           => None,
        }
    }

    fn encode(&self) -> String {
        format!("{:x}.{:x}.{:x}", self.0, self.1, self.2)
    }
}

impl Argument for Coords {

    fn from_nums<T>(mut args: T, default: Option<Coords>) -> Option<Coords>
    where T: Iterator<Item=u32> {
        match (args.next(), args.next()) {
            (Some(x), Some(y))  => Some(Coords {x:x, y:y}),
            _                   => default,
        }
    }

    fn encode(&self) -> String {
        format!("{:x}.{:x}", self.x, self.y)
    }

}

impl Argument for Direction {

    fn from_nums<T>(mut args: T, default: Option<Direction>) -> Option<Direction>
    where T: Iterator<Item=u32> {
        match args.next() {
            Some(1) => Some(Up),
            Some(2) => Some(Down),
            Some(3) => Some(Left),
            Some(4) => Some(Right),
            _       => default
        }
    }

    fn encode(&self) -> String {
        match *self {
            Up      => String::from("1"),
            Down    => String::from("2"),
            Left    => String::from("3"),
            Right   => String::from("4"),
        }
    }

}

impl Argument for InputMode {
    
    fn from_nums<T>(mut args: T, default: Option<InputMode>) -> Option<InputMode>
    where T: Iterator<Item=u32> {
        match args.next() {
            Some(1) => Some(Ansi),
            Some(2) => Some(Extended),
            _       => default,
        }
    }

    fn encode(&self) -> String {
        match *self {
            Ansi | Application  => String::from("1"),
            Extended            => String::from("2"),
        }
    }

}

impl Argument for Movement {

    fn from_nums<T>(mut args: T, default: Option<Movement>) -> Option<Movement>
    where T: Iterator<Item=u32> {
        match args.next() {
            // Position
            Some(0x1)   => Coords::from_nums(args, Some(Coords {x: 0, y: 0})).map(Position),
            // To
            Some(0x2)   => {
                let dir = Direction::from_nums(args.by_ref(), Some(Right)).unwrap();
                let n = u32::from_nums(args.by_ref(), Some(1)).unwrap();
                let wrap = bool::from_nums(args, Some(false)).unwrap();
                Some(To(dir, n, wrap))
            }
            // ToEdge
            Some(0x3)   => Direction::from_nums(args, Some(Right)).map(ToEdge),
            // IndexTo
            Some(0x4)   => {
                let dir = Direction::from_nums(args.by_ref(), Some(Right)).unwrap();
                let n = u32::from_nums(args.by_ref(), Some(1)).unwrap();
                Some(IndexTo(dir, n))
            }
            // Tab
            Some(0x5)   => {
                let dir = Direction::from_nums(args.by_ref(), Some(Right)).unwrap();
                let n = u32::from_nums(args.by_ref(), Some(1)).unwrap();
                let wrap = bool::from_nums(args, Some(false)).unwrap();
                Some(Tab(dir, n, wrap))
            }
            // PreviousLine/NextLine
            Some(0x6)   => {
                let n = u32::from_nums(args.by_ref(), Some(1)).unwrap();
                match bool::from_nums(args, Some(false)).unwrap() {
                    true    => Some(PreviousLine(n)),
                    false   => Some(NextLine(n)),
                }
            }
            // Column
            Some(0x7)   => u32::from_nums(args, Some(0)).map(Column),
            // Row
            Some(0x8)   => u32::from_nums(args, Some(0)).map(Row),
            // ToBeginning/ToEnd
            Some(0x9)   => {
                match bool::from_nums(args, Some(false)).unwrap() {
                    true    => Some(ToBeginning),
                    false   => Some(ToEnd),
                }
            }
            _                   => default,
        }
    }

    fn encode(&self) -> String {
        match *self {
            Position(coords)    => format!("1.{}", coords.encode()),
            To(dir, n, wrap)    => format!("2.{}.{:x}.{}", dir.encode(), n, wrap.encode()),
            ToEdge(dir)         => format!("3.{}", dir.encode()),
            IndexTo(dir, n)     => format!("4.{}.{:x}", dir.encode(), n),
            Tab(dir, n, wrap)   => format!("5.{}.{:x}.{}", dir.encode(), n, wrap.encode()),
            PreviousLine(n)     => format!("6.{:x}.1", n),
            NextLine(n)         => format!("6.{:x}", n),
            Column(n)           => format!("7.{:x}", n),
            Row(n)              => format!("8.{:x}", n),
            ToBeginning         => String::from("9.1"),
            ToEnd               => String::from("9"),
        }
    }

}

impl Argument for Region {

    fn from_nums<T>(mut args: T, default: Option<Region>) -> Option<Region>
    where T: Iterator<Item=u32> {
        match (args.next(), args.next(), args.next(), args.next()) {
            (Some(l), Some(t), Some(r), Some(b)) => Some(Region::new(l, t, r, b)),
            _                                    => None
        }
    }

    fn encode(&self) -> String {
        format!("{:x}.{:x}.{:x}.{:x}", self.left, self.top, self.right, self.bottom)
    }

}

impl Argument for Style {
    
    fn from_nums<T>(mut args: T, default: Option<Style>) -> Option<Style>
    where T: Iterator<Item=u32> {
        match args.next() {
            Some(0x1)   => match args.next() {
                Some(0)         => Some(Underline(0)),
                Some(1) | None  => Some(Underline(1)),
                Some(2)         => Some(Underline(2)),
                _               => None
            },
            Some(0x2)   => bool::from_nums(args, Some(true)).map(Bold),
            Some(0x3)   => bool::from_nums(args, Some(true)).map(Italic),
            Some(0x4)   => bool::from_nums(args, Some(true)).map(Blink),
            Some(0x5)   => bool::from_nums(args, Some(true)).map(InvertColors),
            Some(0x6)   => bool::from_nums(args, Some(true)).map(Strikethrough),
            Some(0x7)   => Some(Opacity(args.next().unwrap_or(0xff) as u8)),
            Some(0x8)   => Color::from_nums(args, None).map(FgColor),
            Some(0x9)   => Color::from_nums(args, None).map(BgColor),
            Some(0xa)   => Some(FgColorCfg(args.next().map(|x| x as u8))),
            Some(0xb)   => Some(BgColorCfg(args.next().map(|x| x as u8))),
            _           => default
        }
    }

    fn encode(&self) -> String {
        match *self {
            Underline(n)        => format!("1.{:x}", n),
            Bold(flag)          => format!("2.{}", flag.encode()),
            Italic(flag)        => format!("3.{}", flag.encode()),
            Blink(flag)         => format!("4.{}", flag.encode()),
            InvertColors(flag)  => format!("5.{}", flag.encode()),
            Strikethrough(flag) => format!("6.{}", flag.encode()),
            Opacity(n)          => format!("7.{:x}", n),
            FgColor(color)      => format!("8.{}", color.encode()),
            BgColor(color)      => format!("9.{}", color.encode()),
            FgColorCfg(None)    => format!("a"),
            FgColorCfg(Some(n)) => format!("a.{:x}", n),
            BgColorCfg(None)    => format!("b"),
            BgColorCfg(Some(n)) => format!("b.{:x}", n),
        }
    }

}

impl Argument for u32 {
    fn from_nums<T>(mut args: T, default: Option<u32>) -> Option<u32>
    where T: Iterator<Item=u32> {
        args.next().or(default)
    }
    fn encode(&self) -> String {
        format!("{:x}", self)
    }
}

#[cfg(test)]
mod tests {

    use datatypes::args::*;
    use super::Argument;

    static AREA_TESTS: &'static [(Area, &'static str)] = &[
        (CursorCell, "1"),
        (CursorRow, "2"),
        (CursorColumn, "3"),
        (CursorTo(To(Up, 2, false)), "4.2.1.2.0"),
        (CursorBound(Coords { x: 0, y: 0 }), "5.0.0"),
        (WholeScreen, "6"),
        (Bound(Region { left: 0, top: 0, right: 0x100, bottom: 0x100 }), "6.0.0.100.100"),
        (Rows(0xff, 0xfff), "7.ff.fff"),
        (Columns(0, 0x10), "8.0.10"),
        (BelowCursor(true), "9.1"),
    ];

    static MOVEMENT_TESTS: &'static [(Movement, &'static str)] = &[
        (Position(Coords { x: 0, y: 0 }), "1.0.0"),
        (To(Up, 0x100, false), "2.1.100.0"),
        (ToEdge(Up), "3.1"),
        (To(Down, 0x1b, false), "2.2.1b.0"),
        (ToEdge(Down), "3.2"),
        (To(Left, 2, false), "2.3.2.0"),
        (ToEdge(Left), "3.3"),
        (To(Right, 1, true), "2.4.1.1"),
        (ToEdge(Right), "3.4"),
        (IndexTo(Up, 1), "4.1.1"),
        (IndexTo(Down, 2), "4.2.2"),
        (IndexTo(Left, 0xfff), "4.3.fff"),
        (IndexTo(Right, 0x10), "4.4.10"),
        (Tab(Left, 1, false), "5.3.1.0"),
        (Tab(Right, 6, false), "5.4.6.0"),
        (PreviousLine(1), "6.1.1"),
        (NextLine(0xf), "6.f"),
        (Column(0), "7.0"),
        (Row(1), "8.1"),
        (ToBeginning, "9.1"),
        (ToEnd, "9"),
    ];

    static STYLE_TESTS: &'static [(Style, &'static str)] = &[
        (Style::Underline(1), "1.1"),
        (Style::Bold(true), "2.1"),
        (Style::Italic(false), "3.0"),
        (Style::Blink(false), "4.0"),
        (Style::InvertColors(false), "5.0"),
        (Style::Strikethrough(true), "6.1"),
        (Style::Opacity(0x40), "7.40"),
        (Style::FgColor(Color(0, 1, 0x19)), "8.0.1.19"),
        (Style::BgColor(Color(0xff, 0xfe, 0xf)), "9.ff.fe.f"),
        (Style::FgColorCfg(None), "a"),
        (Style::FgColorCfg(Some(7)), "a.7"),
        (Style::BgColorCfg(None), "b"),
        (Style::BgColorCfg(Some(0xf)), "b.f"),
    ];

    #[test]
    fn area_from_arg() {
        for &(area, arg) in AREA_TESTS {
            assert_eq!(Area::decode(Some(arg), None), Some(area));
        }
    }

    #[test]
    fn area_to_arg() {
        for &(area, arg) in AREA_TESTS {
            assert_eq!(&area.encode(), arg);
        }
    }

    #[test]
    fn bool_from_arg() {
        assert_eq!(bool::decode(Some("0"), None), Some(false));
        assert_eq!(bool::decode(Some("1"), None), Some(true));
        assert_eq!(bool::decode(Some("2"), None), None);
    }

    #[test]
    fn bool_to_arg() {
        assert_eq!(&false.encode(), "0");
        assert_eq!(&true.encode(),  "1");
    }

    #[test]
    fn color_from_arg() {
        assert_eq!(Color::decode(Some("0.1.2"), None), Some(Color(0, 1, 2)));
        assert_eq!(Color::decode(Some("0"), None), None);
    }

    #[test]
    fn color_to_arg() {
        assert_eq!(&Color(0,1,2).encode(), "0.1.2");
    }

    #[test]
    fn coords_from_arg() {
        assert_eq!(Coords::decode(Some("1.2"), None), Some(Coords{x:1, y:2}));
        assert_eq!(Coords::decode(Some("0"), None), None);
    }

    #[test]
    fn coords_to_arg() {
        assert_eq!(&Coords{x:1, y:2}.encode(), "1.2");
    }

    #[test]
    fn input_mode_from_arg() {
        assert_eq!(InputMode::decode(Some("1"), None), Some(InputMode::Ansi));
        assert_eq!(InputMode::decode(Some("2"), None), Some(InputMode::Extended));
        assert_eq!(InputMode::decode(Some("3"), None), None);
    }

    #[test]
    fn input_mode_to_arg() {
        assert_eq!(&InputMode::Ansi.encode(), "1");
        assert_eq!(&InputMode::Extended.encode(), "2");
    }

    #[test]
    fn movement_from_arg() {
        for &(movement, arg) in MOVEMENT_TESTS {
            assert_eq!(Movement::decode(Some(arg), None), Some(movement));
        }
    }

    #[test]
    fn movement_to_arg() {
        for &(movement, arg) in MOVEMENT_TESTS {
            assert_eq!(&movement.encode(), arg);
        }
    }

    #[test]
    fn region_from_arg() {
        assert_eq!(Region::decode(Some("0.1.2.3"), None), Some(Region::new(0,1,2,3)));
        assert_eq!(Region::decode(Some("0.1.2"), None), None);
    }

    #[test]
    fn region_to_arg() {
        assert_eq!(&Region::new(0,1,2,3).encode(), "0.1.2.3");
    }

    #[test]
    fn style_from_arg() {
        for &(style, arg) in STYLE_TESTS {
            assert_eq!(Style::decode(Some(arg), None), Some(style));
        }
    }

    #[test]
    fn style_to_arg() {
        for &(style, arg) in STYLE_TESTS {
            assert_eq!(&style.encode(), arg);
        }
    }

}
