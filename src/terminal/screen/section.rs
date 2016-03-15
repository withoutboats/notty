use std::mem;
use std::ops::Index;

use datatypes::{Region, Coords, CoordsIter, SaveGrid, SplitKind, ResizeRule};
use datatypes::SplitKind::*;
use terminal::{CharGrid, CharCell};


use super::panel::Panel;
use super::panel::Panel::*;
use super::stack::Stack;

/// A (rectangular) section of the screen, which contains a stack of panels.
pub struct ScreenSection {
    tag: u64,
    area: Region,
    stack: Stack<Panel>,
}

impl ScreenSection {

    /// Construct a new ScreenSection with a given tag for this area of the screen. It will be
    /// filled with an empty grid.
    pub fn new(tag: u64, area: Region) -> ScreenSection {
        let grid = CharGrid::new(area.width(), area.height(), false, false);
        ScreenSection::with_data(tag, area, Grid(grid))
    }

    fn with_data(tag: u64, area: Region, data: Panel) -> ScreenSection {
        ScreenSection {
            tag: tag,
            area: area,
            stack: Stack::new(data)
        }
    }

    /// Returns true if the top panel in this section is a grid, and false if it is split into
    /// multiple grids.
    pub fn is_grid(&self) -> bool {
        self.stack.top.is_grid()
    }

    // Count the number of visible grids in this section of the screen
    pub fn count_grids(&self) -> usize {
        match self.stack.top {
            Grid(_)                             => 1,
            Split { ref left, ref right, .. }   => left.count_grids() + right.count_grids(),
            _                                   => unreachable!(),
        }
    }

    pub fn area(&self) -> Region {
        self.area
    }

    pub fn top(&self) -> &Panel {
        &self.stack.top
    }

    /// Find the section with this tag.
    pub fn find(&self, tag: u64) -> Option<&ScreenSection> {
        if self.tag == tag { Some(self) }
        else { self.stack.iter().flat_map(|panel| panel.find(tag)).next() }
    }

    /// Find the section with this tag, returning a mutable reference.
    pub fn find_mut(&mut self, tag: u64) -> Option<&mut ScreenSection> {
        if self.tag == tag { Some(self) }
        else { self.stack.iter_mut().flat_map(|panel| panel.find_mut(tag)).next() }
    }

    /// Get the grid associated with this section - panic if this section is split.
    pub fn grid(&self) -> &CharGrid {
        match self.stack.top {
            Grid(ref grid) => grid,
            _ => panic!("Cannot call grid on a split section of the screen"),
        }
    }

    /// Get a mutable reference to the grid associated with this section - panic if this section
    /// is split.
    pub fn grid_mut(&mut self) -> &mut CharGrid {
        match self.stack.top {
            Grid(ref mut grid) => grid,
            _ => panic!("Cannot call grid_mut on a split section of the screen"),
        }
    }

    /// Iterate over all of the cells in this section of the screen.
    pub fn cells(&self) -> super::Cells {
        super::Cells {
            iter: CoordsIter::from_region(self.area),
            screen: self,
        }
    }

    /// Iterate over all of the visible leaf panels in this section of the screen.
    pub fn panels(&self) -> super::Panels {
        super::Panels {
            stack: vec![self],
        }
    }

    /// Adjust this section of the grid to fit a new area.
    pub fn resize(&mut self, new_area: Region, rule: ResizeRule) {
        self.area = new_area;
        for panel in &mut self.stack {
            panel.resize(self.area, new_area, rule);
        }
    }

    /// Split the top panel this section into two sections.
    pub fn split(&mut self, save: SaveGrid, kind: SplitKind, rule: ResizeRule,
                 l_tag: u64, r_tag: u64) {
        let (kind, l_area, r_area) = self.area.split(kind, rule);
        match save {
            SaveGrid::Left => {
                let mut l_panel = mem::replace(&mut self.stack.top, DeadGrid);
                l_panel.resize(self.area, l_area, rule);
                self.stack.top = Split {
                    kind: kind,
                    left: Box::new(ScreenSection::with_data(l_tag, l_area, l_panel)),
                    right: Box::new(ScreenSection::new(r_tag, r_area)),
                }
            }
            SaveGrid::Right => {
                let mut r_panel = mem::replace(&mut self.stack.top, DeadGrid);
                r_panel.resize(self.area, r_area, rule);
                self.stack.top = Split {
                    kind: kind,
                    left: Box::new(ScreenSection::new(l_tag, l_area)),
                    right: Box::new(ScreenSection::with_data(r_tag, r_area, r_panel)),
                }
            }
        }
    }

    /// Remove the split in the top panel of this section.
    pub fn unsplit(&mut self, save: SaveGrid) {
        let (mut saved_stack, old_area) = match (save, &mut self.stack.top) {
            (SaveGrid::Left, &mut Split { ref mut left, .. }) => {
                (mem::replace(&mut left.stack, Stack::new(DeadGrid)), left.area)
            }
            (SaveGrid::Right, &mut Split { ref mut right, .. }) => {
                (mem::replace(&mut right.stack, Stack::new(DeadGrid)), right.area)
            }
            _ => return
        };
        for panel in &mut saved_stack {
            panel.resize(old_area, self.area, ResizeRule::Percentage);
        }
        self.stack.extend(saved_stack.into_iter().rev());
    }

    /// Push a new empty grid panel on top of this section.
    pub fn push(&mut self) {
        let grid = CharGrid::new(self.area.width(), self.area.height(), false, false);
        self.stack.push(Grid(grid));
    }

    /// Remove the top panel of this section.
    pub fn pop(&mut self) {
        self.stack.pop();
    }

}

impl Index<Coords> for ScreenSection {
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

#[cfg(test)]
mod tests {

    mod splits {
        use super::super::*;
        use super::super::panel::Panel::*;
        use super::super::ScreenSection;

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
