use std::mem;
use std::ops::Index;

use datatypes::{Region, Coords, CoordsIter, SaveGrid, SplitKind, ResizeRule};
use datatypes::SplitKind::*;
use terminal::{CharGrid, CharCell};

use super::GridFill;
use super::panel::Panel;
use super::panel::Panel::*;
use super::stack::Stack;

/// A (rectangular) section of the screen, which contains a stack of panels.
#[derive(Debug, Eq, PartialEq)]
pub struct ScreenSection<T=CharGrid> where T: GridFill {
    tag: u64,
    area: Region,
    stack: Stack<Panel<T>>,
}

impl<T: GridFill> ScreenSection<T> {

    /// Construct a new ScreenSection with a given tag for this area of the screen. It will be
    /// filled with an empty grid.
    pub fn new(tag: u64, area: Region) -> ScreenSection<T> {
        let grid = T::new(area.width(), area.height(), false);
        ScreenSection::with_data(tag, area, Grid(grid))
    }

    fn with_data(tag: u64, area: Region, data: Panel<T>) -> ScreenSection<T> {
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

    pub fn top(&self) -> &Panel<T> {
        &self.stack.top
    }

    /// Find the section with this tag.
    pub fn find(&self, tag: u64) -> Option<&ScreenSection<T>> {
        if self.tag == tag { Some(self) }
        else { self.stack.iter().flat_map(|panel| panel.find(tag)).next() }
    }

    /// Find the section with this tag, returning a mutable reference.
    pub fn find_mut(&mut self, tag: u64) -> Option<&mut ScreenSection<T>> {
        if self.tag == tag { Some(self) }
        else { self.stack.iter_mut().flat_map(|panel| panel.find_mut(tag)).next() }
    }

    /// Get the grid associated with this section - panic if this section is split.
    pub fn grid(&self) -> &T {
        match self.stack.top {
            Grid(ref grid) => grid,
            _ => panic!("Cannot call grid on a split section of the screen"),
        }
    }

    /// Get a mutable reference to the grid associated with this section - panic if this section
    /// is split.
    pub fn grid_mut(&mut self) -> &mut T {
        match self.stack.top {
            Grid(ref mut grid) => grid,
            _ => panic!("Cannot call grid_mut on a split section of the screen"),
        }
    }

    /// Adjust this section of the grid to fit a new area.
    pub fn resize(&mut self, new_area: Region, rule: ResizeRule) {
        for panel in &mut self.stack {
            panel.resize(self.area, new_area, rule);
        }
        self.area = new_area;
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
            (SaveGrid::Left, &mut Split { ref mut left, .. }) =>
                (mem::replace(&mut left.stack, Stack::new(DeadGrid)), left.area),
            (SaveGrid::Right, &mut Split { ref mut right, .. }) =>
                (mem::replace(&mut right.stack, Stack::new(DeadGrid)), right.area),
            _ => return
        };
        for panel in &mut saved_stack {
            panel.resize(old_area, self.area, ResizeRule::Percentage);
        }
        if self.stack.len() == 1 {
            self.stack = saved_stack;
        } else {
            self.stack.pop();
            self.stack.extend(saved_stack.into_iter().rev());
        }
    }

    /// Push a new empty grid panel on top of this section.
    pub fn push(&mut self) {
        let grid = T::new(self.area.width(), self.area.height(), false);
        self.stack.push(Grid(grid));
    }

    /// Remove the top panel of this section.
    pub fn pop(&mut self) {
        self.stack.pop();
    }

}

impl ScreenSection {

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
    use std::fmt::Debug;

    use super::*;
    use super::super::GridFill;
    use super::super::panel::Panel::*;
    use super::super::stack::Stack;

    use datatypes::{Region, CoordsIter, SaveGrid, SplitKind, ResizeRule};
    use datatypes::SplitKind::*;
    use terminal::CharGrid;

    fn grid_section<T: GridFill>() -> ScreenSection<T> {
        ScreenSection {
            tag: 0,
            area: Region::new(0, 0, 8, 8),
            stack: Stack::new(Grid(T::new(8, 8, false))),
        }
    }

    fn split_section<T: GridFill>() -> ScreenSection<T> {
        ScreenSection {
            tag: 0,
            area: Region::new(0, 0, 8, 8),
            stack: Stack::new(Split {
                kind: Vertical(4),
                left: Box::new(ScreenSection::new(1, Region::new(0, 0, 4, 8))),
                right: Box::new(ScreenSection::new(2, Region::new(4, 0, 8, 8))),
            }),
        }
    }

    fn stack_section<T: GridFill>() -> ScreenSection<T> {
        let mut section = split_section();
        section.push();
        section
    }

    fn run_test<F, T>(f: F, res: [T; 3])
    where F: Fn(ScreenSection<Region>) -> T, T: PartialEq + Debug {
        assert_eq!(f(grid_section()), res[0]);
        assert_eq!(f(split_section()), res[1]);
        assert_eq!(f(stack_section()), res[2]);
    }

    #[test]
    fn new() {
        assert_eq!(grid_section::<Region>(), ScreenSection::new(0, Region::new(0, 0, 8, 8)));
    }

    #[test]
    fn with_data() {
        assert_eq!(split_section::<Region>(), ScreenSection::with_data(0, Region::new(0, 0, 8, 8),
        Split {
            kind: Vertical(4),
            left: Box::new(ScreenSection::new(1, Region::new(0, 0, 4, 8))),
            right: Box::new(ScreenSection::new(2, Region::new(4, 0, 8, 8))),
        }));
    }

    #[test]
    fn is_grid() {
        run_test(|section| section.is_grid(), [true, false, true]);
    }

    #[test]
    fn count_grids() {
        run_test(|section| section.count_grids(), [1, 2, 1]);
    }

    #[test]
    fn area() {
        run_test(|section| section.area(), [
            Region::new(0, 0, 8, 8),
            Region::new(0, 0, 8, 8),
            Region::new(0, 0, 8, 8),
        ]);
    }

    #[test]
    fn find() {
        run_test(|section| section.find(1).is_some(), [false, true, true]);
    }

    #[test]
    fn find_mut() {
        run_test(|mut section| section.find_mut(1).is_some(), [false, true, true]);
    }

    #[test]
    fn grid() {
        assert_eq!(*grid_section::<Region>().grid(), Region::new(0, 0, 8, 8));
    }

    #[test]
    #[should_panic]
    fn grid_on_split() {
        split_section::<Region>().grid();
    }

    #[test]
    fn grid_mut() {
        assert_eq!(*grid_section::<Region>().grid_mut(), Region::new(0, 0, 8, 8));
    }

    #[test]
    #[should_panic]
    fn grid_mut_on_split() {
        split_section::<Region>().grid_mut();
    }

    #[test]
    fn resize() {
        run_test(|mut section| {
            section.resize(Region::new(0, 0, 6, 6), ResizeRule::Percentage);
            section
        }, [
            ScreenSection::new(0, Region::new(0, 0, 6, 6)),
            ScreenSection {
                tag: 0,
                area: Region::new(0, 0, 6, 6),
                stack: Stack::new(Split {
                    kind: Vertical(3),
                    left: Box::new(ScreenSection::new(1, Region::new(0, 0, 3, 6))),
                    right: Box::new(ScreenSection::new(2, Region::new(3, 0, 6, 6))),
                }),
            },
            ScreenSection {
                tag: 0,
                area: Region::new(0, 0, 6, 6),
                stack: {
                    let mut stack = Stack::new(Split {
                        kind: Vertical(3),
                        left: Box::new(ScreenSection::new(1, Region::new(0, 0, 3, 6))),
                        right: Box::new(ScreenSection::new(2, Region::new(3, 0, 6, 6))),
                    });
                    stack.push(Grid(Region::new(0, 0, 6, 6)));
                    stack
                }
            }
        ]);
    }

    #[test]
    fn split_save_left() {
        run_test(|mut section| {
            section.split(SaveGrid::Left, SplitKind::Horizontal(4), ResizeRule::Percentage, 3, 4);
            section
        }, [
            ScreenSection {
                tag: 0,
                area: Region::new(0, 0, 8, 8),
                stack: Stack::new(Split {
                    kind: Horizontal(4),
                    left: Box::new(ScreenSection::new(3, Region::new(0, 0, 8, 4))),
                    right: Box::new(ScreenSection::new(4, Region::new(0, 4, 8, 8))),
                })
            },
            ScreenSection {
                tag: 0,
                area: Region::new(0, 0, 8, 8),
                stack: Stack::new(Split {
                    kind: Horizontal(4),
                    left: Box::new(ScreenSection {
                        tag: 3,
                        area: Region::new(0, 0, 8, 4),
                        stack: Stack::new(Split {
                            kind: Vertical(4),
                            left: Box::new(ScreenSection::new(1, Region::new(0, 0, 4, 4))),
                            right: Box::new(ScreenSection::new(2, Region::new(4, 0, 8, 4))),
                        })
                    }),
                    right: Box::new(ScreenSection::new(4, Region::new(0, 4, 8, 8))),
                })
            },
            ScreenSection {
                tag: 0,
                area: Region::new(0, 0, 8, 8),
                stack: {
                    let mut stack = Stack::new(Split {
                        kind: Vertical(4),
                        left: Box::new(ScreenSection::new(1, Region::new(0, 0, 4, 8))),
                        right: Box::new(ScreenSection::new(2, Region::new(4, 0, 8, 8))),
                    });
                    stack.push(Split {
                        kind: Horizontal(4),
                        left: Box::new(ScreenSection::new(3, Region::new(0, 0, 8, 4))),
                        right: Box::new(ScreenSection::new(4, Region::new(0, 4, 8, 8))),
                    });
                    stack
                },
            },
        ]);
    }

    #[test]
    fn split_save_right() {
        run_test(|mut section| {
            section.split(SaveGrid::Right, SplitKind::Horizontal(4), ResizeRule::Percentage, 3, 4);
            section
        }, [
            ScreenSection {
                tag: 0,
                area: Region::new(0, 0, 8, 8),
                stack: Stack::new(Split {
                    kind: Horizontal(4),
                    left: Box::new(ScreenSection::new(3, Region::new(0, 0, 8, 4))),
                    right: Box::new(ScreenSection::new(4, Region::new(0, 4, 8, 8))),
                })
            },
            ScreenSection {
                tag: 0,
                area: Region::new(0, 0, 8, 8),
                stack: Stack::new(Split {
                    kind: Horizontal(4),
                    left: Box::new(ScreenSection::new(3, Region::new(0, 0, 8, 4))),
                    right: Box::new(ScreenSection {
                        tag: 4,
                        area: Region::new(0, 4, 8, 8),
                        stack: Stack::new(Split {
                            kind: Vertical(4),
                            left: Box::new(ScreenSection::new(1, Region::new(0, 4, 4, 8))),
                            right: Box::new(ScreenSection::new(2, Region::new(4, 4, 8, 8))),
                        }),
                    }),
                })
            },
            ScreenSection {
                tag: 0,
                area: Region::new(0, 0, 8, 8),
                stack: {
                    let mut stack = Stack::new(Split {
                        kind: Vertical(4),
                        left: Box::new(ScreenSection::new(1, Region::new(0, 0, 4, 8))),
                        right: Box::new(ScreenSection::new(2, Region::new(4, 0, 8, 8))),
                    });
                    stack.push(Split {
                        kind: Horizontal(4),
                        left: Box::new(ScreenSection::new(3, Region::new(0, 0, 8, 4))),
                        right: Box::new(ScreenSection::new(4, Region::new(0, 4, 8, 8))),
                    });
                    stack
                },
            },
        ]);
    }

    #[test]
    fn unsplit_save_left() {
        run_test(|mut section| { section.unsplit(SaveGrid::Left); section }, [
            grid_section::<Region>(),
            grid_section::<Region>(),
            stack_section::<Region>(),
        ]);
    }

    #[test]
    fn unsplit_save_right() {
        run_test(|mut section| { section.unsplit(SaveGrid::Right); section }, [
            grid_section::<Region>(),
            grid_section::<Region>(),
            stack_section::<Region>(),
        ]);
    }

    #[test]
    fn push() {
        run_test(|mut section| { section.push(); *section.grid() }, [
            Region::new(0, 0, 8, 8),
            Region::new(0, 0, 8, 8),
            Region::new(0, 0, 8, 8),
        ]);
    }

    #[test]
    fn pop() {
        run_test(|mut section| { section.pop(); section }, [
            grid_section::<Region>(),
            split_section::<Region>(),
            split_section::<Region>(),
        ]);
    }

    #[test]
    fn index() {
        for section in &[grid_section::<CharGrid>(), split_section(), stack_section()] {
            let cells = CoordsIter::from_region(Region::new(0, 0, 8, 8))
                            .map(|coords| &section[coords]).collect::<Vec<_>>();
            assert_eq!(cells.len(), 64);
        }
    }

}
