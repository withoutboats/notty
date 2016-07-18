use std::mem;
use std::ops::Index;

use datatypes::{Flow, Region, Coords, CoordsIter, SaveGrid, SplitKind, ResizeRule, GridSettings};
use datatypes::SplitKind::*;
use terminal::interfaces::{ConstructGrid, Resizeable};

use super::panel::Panel;
use super::panel::Panel::*;
use super::ring::Ring;

/// A (rectangular) section of the screen, which contains a ring of panels.
#[derive(Debug, Eq, PartialEq)]
pub struct ScreenSection<T> {
    tag: u64,
    area: Region,
    ring: Ring<Panel<T>>,
}

impl<T: ConstructGrid + Resizeable> ScreenSection<T> {
    /// Split the top panel this section into two sections.
    pub fn split(&mut self, save: SaveGrid, kind: SplitKind, rule: ResizeRule,
                 l_tag: u64, r_tag: u64, retain_offscreen_state: bool) {
        let (kind, l_area, r_area) = self.area.split(kind, rule);
        match save {
            SaveGrid::Left => {
                let mut l_panel = mem::replace(&mut self.ring.top, DeadGrid);
                l_panel.resize(self.area, l_area, rule);
                self.ring.top = Split {
                    kind: kind,
                    left: Box::new(ScreenSection::with_data(l_tag, l_area, l_panel)),
                    right: Box::new(ScreenSection::new(r_tag, r_area, retain_offscreen_state)),
                }
            }
            SaveGrid::Right => {
                let mut r_panel = mem::replace(&mut self.ring.top, DeadGrid);
                r_panel.resize(self.area, r_area, rule);
                self.ring.top = Split {
                    kind: kind,
                    left: Box::new(ScreenSection::new(l_tag, l_area, retain_offscreen_state)),
                    right: Box::new(ScreenSection::with_data(r_tag, r_area, r_panel)),
                }
            }
        }
    }
}

impl<T: ConstructGrid> ScreenSection<T> {

    /// Construct a new ScreenSection with a given tag for this area of the screen. It will be
    /// filled with an empty grid.
    pub fn new(tag: u64, area: Region, retain_offscreen_state: bool)
            -> ScreenSection<T> {
        let grid = T::new(GridSettings {
            width: area.width(),
            height: area.height(),
            retain_offscreen_state: retain_offscreen_state,
            flow: Flow::Moveable,
        });
        ScreenSection::with_data(tag, area, Grid(grid))
    }

    /// Push a new empty grid panel on top of this section.
    pub fn push(&mut self, retain_offscreen_state: bool) {
        let grid = T::new(GridSettings {
            width: self.area.width(),
            height: self.area.height(),
            retain_offscreen_state: retain_offscreen_state,
            flow: Flow::Moveable,
        });
        self.ring.push(Grid(grid));
    }
}

impl<T: Resizeable> ScreenSection<T> {
    /// Adjust this section of the grid to fit a new area.
    pub fn resize(&mut self, new_area: Region, rule: ResizeRule) {
        for panel in &mut self.ring {
            panel.resize(self.area, new_area, rule);
        }
        self.area = new_area;
    }

    pub fn adjust_split(&mut self, new_kind: SplitKind, rule: ResizeRule) {
        if let Split { ref mut left, ref mut right, ref mut kind, .. } = self.ring.top {
            let (new_kind, l_area, r_area) = self.area.split(new_kind, rule);
            *kind = new_kind;
            left.resize(l_area, rule);
            right.resize(r_area, rule);
        }
    }

    /// Remove the split in the top panel of this section.
    pub fn unsplit(&mut self, save: SaveGrid) {
        let (mut saved_ring, old_area) = match (save, &mut self.ring.top) {
            (SaveGrid::Left, &mut Split { ref mut left, .. }) =>
                (mem::replace(&mut left.ring, Ring::new(DeadGrid)), left.area),
            (SaveGrid::Right, &mut Split { ref mut right, .. }) =>
                (mem::replace(&mut right.ring, Ring::new(DeadGrid)), right.area),
            _ => return
        };
        for panel in &mut saved_ring {
            panel.resize(old_area, self.area, ResizeRule::Percentage);
        }
        if self.ring.len() == 1 {
            self.ring = saved_ring;
        } else {
            self.ring.pop();
            self.ring.extend(saved_ring.into_iter().rev());
        }
    }
}

impl<T> ScreenSection<T> {
    fn with_data(tag: u64, area: Region, data: Panel<T>) -> ScreenSection<T> {
        ScreenSection {
            tag: tag,
            area: area,
            ring: Ring::new(data)
        }
    }

    /// Returns true if the top panel in this section is a grid, and false if it is split into
    /// multiple grids.
    pub fn is_grid(&self) -> bool {
        self.ring.top.is_grid()
    }

    // Count the number of visible grids in this section of the screen
    pub fn count_grids(&self) -> usize {
        match self.ring.top {
            Grid(_)                             => 1,
            Split { ref left, ref right, .. }   => left.count_grids() + right.count_grids(),
            _                                   => unreachable!(),
        }
    }

    pub fn area(&self) -> Region {
        self.area
    }

    pub fn tag(&self) -> u64 {
        self.tag
    }

    pub fn top(&self) -> &Panel<T> {
        &self.ring.top
    }

    /// Find the section with this tag.
    pub fn find(&self, tag: u64) -> Option<&ScreenSection<T>> {
        if self.tag == tag { Some(self) }
        else { self.ring.iter().flat_map(|panel| panel.find(tag)).next() }
    }

    /// Find the section with this tag, returning a mutable reference.
    pub fn find_mut(&mut self, tag: u64) -> Option<&mut ScreenSection<T>> {
        if self.tag == tag { Some(self) }
        else { self.ring.iter_mut().flat_map(|panel| panel.find_mut(tag)).next() }
    }

    /// Get the grid associated with this section - panic if this section is split.
    pub fn grid(&self) -> &T {
        match self.ring.top {
            Grid(ref grid) => grid,
            _ => panic!("Cannot call grid on a split section of the screen"),
        }
    }

    /// Get a mutable reference to the grid associated with this section - panic if this section
    /// is split.
    pub fn grid_mut(&mut self) -> &mut T {
        match self.ring.top {
            Grid(ref mut grid) => grid,
            _ => panic!("Cannot call grid_mut on a split section of the screen"),
        }
    }

    /// Get a reference to the children section of this section if it is split.
    pub fn children(&self) -> Option<(&ScreenSection<T>, &ScreenSection<T>)> {
        if let Split { ref left, ref right, .. } = self.ring.top {
            Some((left, right))
        } else { None }
    }

    /// Remove the top panel of this section.
    pub fn pop(&mut self) {
        self.ring.pop();
    }

    pub fn rotate_down(&mut self) {
        self.ring.rotate_down();
    }

    pub fn rotate_up(&mut self) {
        self.ring.rotate_up();
    }

    /// Iterate over all of the cells in this section of the screen.
    pub fn cells(&self) -> super::Cells<T> {
        super::Cells {
            iter: CoordsIter::from_region(self.area),
            section: self,
        }
    }

    /// Iterate over all of the visible leaf panels in this section of the screen.
    pub fn panels(&self) -> super::Panels<T> {
        super::Panels {
            stack: vec![self],
        }
    }

}

impl<T: Index<Coords>> Index<Coords> for ScreenSection<T> {
    type Output = T::Output;
    fn index(&self, Coords { x, y }: Coords) -> &Self::Output {
        match self.ring.top {
            Grid(ref grid)                                          => {
                &grid[Coords { x: x, y: y }]
            }
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
    pub use terminal::screen::tests::*;

    fn split(mut section: ScreenSection<MockGrid>, save: SaveGrid) -> ScreenSection<MockGrid> {
        section.split(save, Horizontal(4), Percentage, 3, 4, false);
        section
    }

    mod grid {
        use super::*;

        pub fn section() -> ScreenSection<MockGrid> {
            ScreenSection::with_data(0, Region::new(0, 0, 8, 8), Grid(GRID))
        }

        #[test]
        fn split_save_left() {
            assert_eq!(super::split(section(), SaveGrid::Left), ScreenSection {
                tag: 0,
                area: Region::new(0, 0, 8, 8),
                ring: Ring::new(Split {
                    kind: Horizontal(4),
                    left: Box::new(ScreenSection::new(3, Region::new(0, 0, 8, 4), false)),
                    right: Box::new(ScreenSection::new(4, Region::new(0, 4, 8, 8), false)),
                })
            })
        }

        #[test]
        fn split_save_right() {
            assert_eq!(super::split(section(), SaveGrid::Right), ScreenSection {
                tag: 0,
                area: Region::new(0, 0, 8, 8),
                ring: Ring::new(Split {
                    kind: Horizontal(4),
                    left: Box::new(ScreenSection::new(3, Region::new(0, 0, 8, 4), false)),
                    right: Box::new(ScreenSection::new(4, Region::new(0, 4, 8, 8), false)),
                })
            })
        }

        #[test]
        fn is_grid() {
            assert!(section().is_grid())
        }

        #[test]
        fn count_grids() {
            assert_eq!(section().count_grids(), 1);
        }
    }

    mod split {
        use super::*;

        pub fn section() -> ScreenSection<MockGrid> {
            ScreenSection {
                tag: 0,
                area: Region::new(0, 0, 8, 8),
                ring: Ring::new(Split {
                    kind: Vertical(4),
                    left: Box::new(ScreenSection::new(1, Region::new(0, 0, 4, 8), false)),
                    right: Box::new(ScreenSection::new(2, Region::new(4, 0, 8, 8), false)),
                }),
            }
        }

        #[test]
        fn split_save_left() {
            assert_eq!(super::split(section(), SaveGrid::Left), ScreenSection {
                tag: 0,
                area: Region::new(0, 0, 8, 8),
                ring: Ring::new(Split {
                    kind: Horizontal(4),
                    left: Box::new(ScreenSection {
                        tag: 3,
                        area: Region::new(0, 0, 8, 4),
                        ring: Ring::new(Split {
                            kind: Vertical(4),
                            left: Box::new(ScreenSection::new(1, Region::new(0, 0, 4, 4), false)),
                            right: Box::new(ScreenSection::new(2, Region::new(4, 0, 8, 4), false)),
                        })
                    }),
                    right: Box::new(ScreenSection::new(4, Region::new(0, 4, 8, 8), false)),
                })
            })
        }

        #[test]
        fn split_save_right() {
            assert_eq!(super::split(section(), SaveGrid::Right), ScreenSection {
                tag: 0,
                area: Region::new(0, 0, 8, 8),
                ring: Ring::new(Split {
                    kind: Horizontal(4),
                    left: Box::new(ScreenSection::new(3, Region::new(0, 0, 8, 4), false)),
                    right: Box::new(ScreenSection {
                        tag: 4,
                        area: Region::new(0, 4, 8, 8),
                        ring: Ring::new(Split {
                            kind: Vertical(4),
                            left: Box::new(ScreenSection::new(1, Region::new(0, 4, 4, 8), false)),
                            right: Box::new(ScreenSection::new(2, Region::new(4, 4, 8, 8), false)),
                        }),
                    }),
                })
            })
        }

        #[test]
        fn is_grid() {
            assert!(!section().is_grid())
        }

        #[test]
        fn count_grids() {
            assert_eq!(section().count_grids(), 2);
        }
    }

    mod ring {
        use super::*;

        pub fn section() -> ScreenSection<MockGrid> {
            let mut section = super::split::section();
            section.push(false);
            section
        }

        #[test]
        fn split_save_left() {
            assert_eq!(super::split(section(), SaveGrid::Left), ScreenSection {
                tag: 0,
                area: Region::new(0, 0, 8, 8),
                ring: {
                    let mut ring = Ring::new(Split {
                        kind: Vertical(4),
                        left: Box::new(ScreenSection::new(1, Region::new(0, 0, 4, 8), false)),
                        right: Box::new(ScreenSection::new(2, Region::new(4, 0, 8, 8), false)),
                    });
                    ring.push(Split {
                        kind: Horizontal(4),
                        left: Box::new(ScreenSection::new(3, Region::new(0, 0, 8, 4), false)),
                        right: Box::new(ScreenSection::new(4, Region::new(0, 4, 8, 8), false)),
                    });
                    ring
                },
            })
        }

        #[test]
        fn split_save_right() {
            assert_eq!(super::split(section(), SaveGrid::Right), ScreenSection {
                tag: 0,
                area: Region::new(0, 0, 8, 8),
                ring: {
                    let mut ring = Ring::new(Split {
                        kind: Vertical(4),
                        left: Box::new(ScreenSection::new(1, Region::new(0, 0, 4, 8), false)),
                        right: Box::new(ScreenSection::new(2, Region::new(4, 0, 8, 8), false)),
                    });
                    ring.push(Split {
                        kind: Horizontal(4),
                        left: Box::new(ScreenSection::new(3, Region::new(0, 0, 8, 4), false)),
                        right: Box::new(ScreenSection::new(4, Region::new(0, 4, 8, 8), false)),
                    });
                    ring
                },
            })
        }

        #[test]
        fn is_grid() {
            assert!(section().is_grid())
        }

        #[test]
        fn count_grids() {
            assert_eq!(section().count_grids(), 1);
        }
    }

    #[cfg(never)]
    mod foo {

    #[test]
    fn new() {
        assert_eq!(grid_section(), ScreenSection::new(0, Region::new(0, 0, 8, 8), true));
    }

    #[test]
    fn with_data() {
        assert_eq!(split_section(), ScreenSection::with_data(0, Region::new(0, 0, 8, 8),
        Split {
            kind: Vertical(4),
            left: Box::new(ScreenSection::new(1, Region::new(0, 0, 4, 8), false)),
            right: Box::new(ScreenSection::new(2, Region::new(4, 0, 8, 8), false)),
        }));
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
        assert_eq!(*grid_section().grid(), Region::new(0, 0, 8, 8));
    }

    #[test]
    #[should_panic]
    fn grid_on_split() {
        split_section().grid();
    }

    #[test]
    fn grid_mut() {
        assert_eq!(*grid_section().grid_mut(), Region::new(0, 0, 8, 8));
    }

    #[test]
    #[should_panic]
    fn grid_mut_on_split() {
        split_section().grid_mut();
    }

    #[test]
    fn resize() {
        run_test(|mut section| {
            section.resize(Region::new(0, 0, 6, 6), ResizeRule::Percentage);
            section
        }, [
            ScreenSection::new(0, Region::new(0, 0, 6, 6), false),
            ScreenSection {
                tag: 0,
                area: Region::new(0, 0, 6, 6),
                ring: Ring::new(Split {
                    kind: Vertical(3),
                    left: Box::new(ScreenSection::new(1, Region::new(0, 0, 3, 6), false)),
                    right: Box::new(ScreenSection::new(2, Region::new(3, 0, 6, 6), false)),
                }),
            },
            ScreenSection {
                tag: 0,
                area: Region::new(0, 0, 6, 6),
                ring: {
                    let mut ring = Ring::new(Split {
                        kind: Vertical(3),
                        left: Box::new(ScreenSection::new(1, Region::new(0, 0, 3, 6), false)),
                        right: Box::new(ScreenSection::new(2, Region::new(3, 0, 6, 6), false)),
                    });
                    ring.push(Grid(Region::new(0, 0, 6, 6)));
                    ring
                }
            }
        ]);
    }

    #[test]
    fn unsplit_save_left() {
        run_test(|mut section| { section.unsplit(SaveGrid::Left); section }, [
            grid_section(),
            grid_section(),
            ring_section(),
        ]);
    }

    #[test]
    fn unsplit_save_right() {
        run_test(|mut section| { section.unsplit(SaveGrid::Right); section }, [
            grid_section(),
            grid_section(),
            ring_section(),
        ]);
    }

    #[test]
    fn adjust_split() {
        run_test(|mut section| {
            section.adjust_split(Horizontal(4), ResizeRule::Percentage);
            section
        }, [
            grid_section(),
            ScreenSection {
                tag: 0,
                area: Region::new(0, 0, 8, 8),
                ring: Ring::new(Split {
                    kind: Horizontal(4),
                    left: Box::new(ScreenSection::new(1, Region::new(0, 0, 8, 4), false)),
                    right: Box::new(ScreenSection::new(2, Region::new(0, 4, 8, 8), false)),
                }),
            },
            ring_section(),
        ]);
    }

    #[test]
    fn push() {
        run_test(|mut section| { section.push(false); *section.grid() }, [
            Region::new(0, 0, 8, 8),
            Region::new(0, 0, 8, 8),
            Region::new(0, 0, 8, 8),
        ]);
    }

    #[test]
    fn pop() {
        run_test(|mut section| { section.pop(); section }, [
            grid_section(),
            split_section(),
            split_section(),
        ]);
    }

    #[test]
    fn index() {
        for section in &[grid_section(), split_section(), ring_section()] {
            let cells = CoordsIter::from_region(Region::new(0, 0, 8, 8))
                            .map(|coords| &section[coords]).collect::<Vec<_>>();
            assert_eq!(cells.len(), 64);
        }
    }

    }
}
