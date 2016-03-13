use std::cmp;
use std::mem;
use std::ops::Index;

use datatypes::{Region, Coords};

use terminal::{CharGrid, CharCell};
use super::Stack;

use self::PanelKind::*;
use self::SplitKind::*;
use self::ResizeRule::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SplitKind {
    Horizontal(u32),
    Vertical(u32),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SaveGrid {
    Left, Right
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ResizeRule {
    Percentage,
    MaxLeftTop,
    MaxRightBottom,
}

pub struct Panel {
    pub area: Region,
    tag: u64,
    stack: Stack<PanelKind>,
}

impl Panel {

    pub fn new(tag: u64, area: Region) -> Panel {
        let grid = CharGrid::new(area.width(), area.height(), false, false);
        Panel::with_data(tag, area, Grid(grid))
    }

    fn with_data(tag: u64, area: Region, data: PanelKind) -> Panel {
        Panel {
            area: area,
            tag: tag,
            stack: Stack::new(data),
        }
    }

    pub fn is_grid(&self) -> bool { 
        self.stack.top.is_grid()
    }

    pub fn find(&self, tag: u64) -> Option<&Panel> {
        if self.tag == tag { Some(self) }
        else { self.stack.iter().flat_map(|panel| panel.find(tag)).next() }
    }

    pub fn find_mut(&mut self, tag: u64) -> Option<&mut Panel> {
        if self.tag == tag { Some(self) }
        else { self.stack.iter_mut().flat_map(|panel| panel.find_mut(tag)).next() }
    }

    pub fn grid(&self) -> &CharGrid {
        match self.stack.top {
            Grid(ref grid) => grid,
            _ => panic!("Cannot call grid on a split panel"),
        }
    }

    pub fn grid_mut(&mut self) -> &mut CharGrid {
        match self.stack.top {
            Grid(ref mut grid) => grid,
            _ => panic!("Cannot call grid_mut on a split panel"),
        }
    }

    pub fn resize(&mut self, new_a: Region, rule: ResizeRule) {
        self.area = new_a;
        for panel in &mut self.stack {
            panel.resize(self.area, new_a, rule);
        }
    }

    pub fn split(&mut self, save: SaveGrid, kind: SplitKind, rule: ResizeRule,
                 l_tag: u64, r_tag: u64) {
        let (kind, l_region, r_region) = split_region(self.area, kind, rule);
        match save {
            SaveGrid::Left  => {
                let mut left = mem::replace(&mut self.stack.top, DeadGrid);
                left.resize(self.area, l_region, rule);
                self.stack.top = Split {
                    kind: kind,
                    left: Box::new(Panel::with_data(l_tag, l_region, left)),
                    right: Box::new(Panel::new(r_tag, r_region))
                };
            }
            SaveGrid::Right => {
                let mut right = mem::replace(&mut self.stack.top, DeadGrid);
                right.resize(self.area, r_region, rule);
                self.stack.top = Split {
                    kind: kind,
                    left: Box::new(Panel::new(l_tag, l_region)),
                    right: Box::new(Panel::with_data(r_tag, r_region, right)),
                };
            }
        }
    }

    pub fn unsplit(&mut self, save: SaveGrid, rule: ResizeRule) {
        let (mut saved_stack, old_a) = match (save, &mut self.stack.top) {
            (SaveGrid::Left, &mut Split { ref mut left, .. }) =>
                (mem::replace(&mut left.stack, Stack::new(DeadGrid)), left.area),
            (SaveGrid::Right, &mut Split { ref mut right, .. })
                => (mem::replace(&mut right.stack, Stack::new(DeadGrid)), right.area),
            _ => return
        };
        for panel in &mut saved_stack {
            panel.resize(old_a, self.area, rule);
        }
        self.stack.extend(saved_stack.into_iter().rev());
    }

    pub fn push(&mut self) {
        self.stack.push(Grid(CharGrid::new(self.area.width(), self.area.height(), false, false)));
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

}

impl Index<Coords> for Panel {
    type Output = CharCell;
    fn index(&self, Coords { x, y }: Coords) -> &CharCell {
        match self.stack.top {
            Grid(ref grid) => &grid[Coords { x: x, y: y }],
            Split { kind: Horizontal(n), ref left, .. } if y < n    => {
                &left[Coords { x: x, y: y }]
            }
            Split { kind: Vertical(n), ref left, .. } if x < n      => {
                &left[Coords { x: x, y: y }]
            }
            Split { kind: Horizontal(n), ref right, .. } if n <= y  => {
                &right[Coords { x: x, y: y - n }]
            }
            Split { kind: Vertical(n), ref right, .. } if n <= x    => {
                &right[Coords { x: x - n, y: y }]
            }
            _                                                       => unreachable!()
        }
    }
}

enum PanelKind {
    Grid(CharGrid),
    Split {
        kind: SplitKind,
        left: Box<Panel>,
        right: Box<Panel>,
    },
    DeadGrid,
}

impl PanelKind {

    fn is_grid(&self) -> bool {
        if let Grid(_) = *self { true } else { false }
    }

    fn find(&self, tag: u64) -> Option<&Panel> {
        if let Split { ref left, ref right, .. } = *self {
            left.find(tag).or_else(move || right.find(tag))
        } else { None }
    }

    fn find_mut(&mut self, tag: u64) -> Option<&mut Panel> {
        if let Split { ref mut left, ref mut right, .. } = *self {
            left.find_mut(tag).or_else(move || right.find_mut(tag))
        } else { None }
    }

    fn resize(&mut self, old_a: Region, new_a: Region, rule: ResizeRule) {
        match *self {
            Grid(ref mut grid) => {
                grid.resize(new_a);
            }
            Split { ref mut left, ref mut right, ref mut kind } => {
                *kind = match (*kind, rule) {
                    (Horizontal(mut n), Percentage) => {
                        n = (n as f32 / old_a.height() as f32 * new_a.height() as f32) as u32;
                        Horizontal(n)
                    }
                    (Vertical(mut n), Percentage)   => {
                        n = (n as f32 / old_a.width() as f32 * new_a.width() as f32) as u32;
                        Vertical(n)
                    }
                    _                               => *kind
                };
                let (new_kind, l_area, r_area) = split_region(new_a, *kind, rule);
                *kind = new_kind;
                left.resize(l_area, rule);
                right.resize(r_area, rule);
            }
            DeadGrid => unreachable!()
        }
    }

}

fn split_region(region: Region, kind: SplitKind, rule: ResizeRule) -> (SplitKind, Region, Region) {
    match (kind, rule) {
        (Horizontal(n), MaxLeftTop) | (Horizontal(n), Percentage)   => {
            let n = cmp::min(region.top + n, region.bottom - 1);
            (Horizontal(n), Region { bottom: n, ..region }, Region { top: n, ..region })
        }
        (Horizontal(n), MaxRightBottom)                             => {
            let n = cmp::max(region.bottom.saturating_sub(n), region.top + 1);
            (Horizontal(n), Region { bottom: n, ..region }, Region { top: n, ..region })
        }
        (Vertical(n), MaxLeftTop) | (Vertical(n), Percentage)       => {
            let n = cmp::min(region.left + n, region.right - 1);
            (Vertical(n), Region { right: n, ..region }, Region { left: n, ..region })
        }
        (Vertical(n), MaxRightBottom)                               => {
            let n = cmp::max(region.right.saturating_sub(n), region.left + 1);
            (Vertical(n), Region { right: n, ..region }, Region { left: n, ..region })
        }
    }
}

#[cfg(test)]
mod tests {

    mod splits {
        use super::super::*;
        use super::super::Panel;
        use super::super::PanelKind::*;

        use datatypes::Region;

        // The hierarchy this sets up is:
        //  0
        //  | \
        //  1  2
        //  | \
        //  3 0x0beefdad
        fn setup_panel() -> Panel {
            Split {
                tag: 0,
                area: Region::new(0, 0, 8, 8),
                kind: SplitKind::Vertical(4),
                left: Box::new(Split {
                    tag: 1,
                    area: Region::new(0, 0, 4, 8),
                    kind: SplitKind::Horizontal(4),
                    left: Box::new(Grid(3, Region::new(0, 0, 4, 4))),
                    right: Box::new(Grid(0x0beefdad, Region::new(0, 4, 4, 8))),
                }),
                right: Box::new(Grid(2, Region::new(4, 0, 8, 8))),
            }
        }

        // After this test:
        //  0
        //  | \
        //  1  2
        //  | \
        //  4 0x0badcafe
        //  | \
        //  3 0x0beefdad
        #[test]
        fn split_grid_1() {
            let mut gh = setup_panel();
            gh.find_mut(1).unwrap().split(&mut (), |_, _, _, _| {}, |_, _, _| {}, SaveGrid::Left,
                                          SplitKind::Horizontal(4), ResizeRule::Percentage,
                                          4, 0x0badcafe);
            assert_eq!(gh, Split {
                tag: 0,
                area: Region::new(0, 0, 8, 8),
                kind: SplitKind::Vertical(4),
                left: Box::new(Split {
                    tag: 1,
                    area: Region::new(0,0, 4, 8),
                    kind: SplitKind::Horizontal(4),
                    left: Box::new(Split {
                        tag: 4,
                        area: Region::new(0, 0, 4, 4),
                        kind: SplitKind::Horizontal(2),
                        left: Box::new(Grid(3, Region::new(0, 0, 4, 2))),
                        right: Box::new(Grid(0x0beefdad, Region::new(0, 2, 4, 4))),
                    }),
                    right: Box::new(Grid(0x0badcafe, Region::new(0, 4, 4, 8)))
                }),
                right: Box::new(Grid(2, Region::new(4, 0, 8, 8)))
            });
        }

        // After this test:
        //       0
        //     /    \
        //    1      2
        //  / |      | \
        // 3 beefdad 4 badcafe
        #[test]
        fn split_grid_2() {
            let mut gh = setup_panel();
            gh.find_mut(2).unwrap().split(&mut (), |_, _, _, _| {}, |_, _, _| {}, SaveGrid::Left,
                                          SplitKind::Horizontal(2), ResizeRule::Percentage,
                                          4, 0x0badcafe);
            assert_eq!(gh, Split {
                tag: 0,
                area: Region::new(0, 0, 8, 8),
                kind: SplitKind::Vertical(4),
                left: Box::new(Split {
                    tag: 1,
                    area: Region::new(0, 0, 4, 8),
                    kind: SplitKind::Horizontal(4),
                    left: Box::new(Grid(3, Region::new(0, 0, 4, 4))),
                    right: Box::new(Grid(0x0beefdad, Region::new(0, 4, 4, 8))),
                }),
                right: Box::new(Split {
                    tag: 2,
                    area: Region::new(4, 0, 8, 8),
                    kind: SplitKind::Horizontal(2),
                    left: Box::new(Grid(4, Region::new(4, 0, 8, 2))),
                    right: Box::new(Grid(0x0badcafe, Region::new(4, 2, 8, 8))),
                }),
            })
        }

        // After this test:
        //  0
        //  | \
        //  1  2
        //  | \
        //  3 0x0beefdad
        //  | \
        //  4 0x0badcafe
        #[test]
        fn split_grid_3() {
            let mut gh = setup_panel();
            gh.find_mut(3).unwrap().split(&mut (), |_, _, _, _| {}, |_, _, _| {}, SaveGrid::Right,
                                          SplitKind::Vertical(6), ResizeRule::MaxLeftTop,
                                          4, 0x0badcafe);
            assert_eq!(gh, Split {
                tag: 0,
                area: Region::new(0, 0, 8, 8),
                kind: SplitKind::Vertical(4),
                left: Box::new(Split {
                    tag: 1,
                    area: Region::new(0,0, 4, 8),
                    kind: SplitKind::Horizontal(4),
                    left: Box::new(Split {
                        tag: 3,
                        area: Region::new(0, 0, 4, 4),
                        kind: SplitKind::Vertical(3),
                        left: Box::new(Grid(4, Region::new(0, 0, 3, 4))),
                        right: Box::new(Grid(0x0badcafe, Region::new(3, 0, 4, 4))),
                    }),
                    right: Box::new(Grid(0x0beefdad, Region::new(0, 4, 4, 8)))
                }),
                right: Box::new(Grid(2, Region::new(4, 0, 8, 8)))
            })
        }

        // After this test:
        // 0
        // | \
        // 3  2
        #[test]
        fn remove_a_grid_beefdad() {
            let mut gh = setup_panel();
            gh.remove(&mut (), |_, _| {}, |_, _, _| {}, 0x0beefdad, ResizeRule::Percentage);
            assert_eq!(gh, Split {
                tag: 0,
                area: Region::new(0, 0, 8, 8),
                kind: SplitKind::Vertical(4),
                left: Box::new(Grid(3, Region::new(0, 0, 4, 8))),
                right: Box::new(Grid(2, Region::new(4, 0, 8, 8))),
            })
        }
    
        // After this test:
        // 2
        #[test]
        fn remove_grid_1() {
            let mut gh = setup_panel();
            gh.remove(&mut (), |_, _| {}, |_, _, _| {}, 1, ResizeRule::Percentage);
            assert_eq!(gh, Grid(2, Region::new(0, 0, 8, 8)));
        }
    
        // After this test:
        // 1
        // | \
        // 3  0x0beefdad
        #[test]
        fn remove_grid_2() {
            let mut gh = setup_panel();
            gh.remove(&mut (), |_, _| {}, |_, _, _| {}, 2, ResizeRule::Percentage);
            assert_eq!(gh, Split {
                tag: 1,
                area: Region::new(0, 0, 8, 8),
                kind: SplitKind::Horizontal(4),
                left: Box::new(Grid(3, Region::new(0, 0, 8, 4))),
                right: Box::new(Grid(0x0beefdad, Region::new(0, 4, 8, 8))),
            })
        }
    }
}
