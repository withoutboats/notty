use std::mem;
use std::ops::Index;

use datatypes::{Flow, Region, Coords, SaveGrid, SplitKind, ResizeRule, GridSettings};
use terminal::interfaces::{ConstructGrid, Resizeable};

use super::panel::Panel;
use super::panel::Panel::*;
use super::ring::Ring;
use super::split::SplitSection;

/// A (rectangular) section of the screen, which contains a ring of panels.
#[derive(Debug, Eq, PartialEq)]
pub struct ScreenSection<T> {
    tag: u64,
    area: Region,
    pub ring: Ring<Panel<T>>,
}

impl<T: ConstructGrid + Resizeable> ScreenSection<T> {
    /// Split the top panel this section into two sections.
    pub fn split(&mut self, save: SaveGrid, kind: SplitKind, rule: ResizeRule,
                 l_tag: u64, r_tag: u64, retain_offscreen_state: bool) {
        let (kind, l_area, r_area) = self.area.split(kind, rule);
        match save {
            SaveGrid::Left => {
                let mut l_panel = mem::replace(&mut self.ring.top, Dead);
                l_panel.shift_into(l_area);
                self.ring.top = Split(SplitSection::new(
                    Box::new(ScreenSection::with_data(l_tag, l_area, l_panel)),
                    Box::new(ScreenSection::new(r_tag, r_area, retain_offscreen_state)),
                    self.area,
                    kind,
                ));
            }
            SaveGrid::Right => {
                let mut r_panel = mem::replace(&mut self.ring.top, Dead);
                r_panel.shift_into(r_area);
                self.ring.top = Split(SplitSection::new(
                    Box::new(ScreenSection::new(l_tag, l_area, retain_offscreen_state)),
                    Box::new(ScreenSection::with_data(r_tag, r_area, r_panel)),
                    self.area,
                    kind,
                ));
            }
        }
    }
}

impl<T: ConstructGrid> ScreenSection<T> {

    /// Construct a new ScreenSection with a given tag for this area of the screen. It will be
    /// filled with an empty grid.
    pub fn new(tag: u64, area: Region, retain_offscreen_state: bool)
            -> ScreenSection<T> {
        let fill = T::new(GridSettings {
            width: area.width(),
            height: area.height(),
            retain_offscreen_state: retain_offscreen_state,
            flow: Flow::Moveable,
        });
        ScreenSection::with_data(tag, area, Fill(fill))
    }

    /// Push a new empty grid panel on top of this section.
    pub fn push(&mut self, retain_offscreen_state: bool) {
        let fill = T::new(GridSettings {
            width: self.area.width(),
            height: self.area.height(),
            retain_offscreen_state: retain_offscreen_state,
            flow: Flow::Moveable,
        });
        self.ring.push(Fill(fill));
    }
}

impl<T: Resizeable> ScreenSection<T> {
    pub fn shift_into(&mut self, area: Region) {
        self.area = area;
        self.resize(area.width(), area.height());
    }

    /// Adjust the split in the top panel of this section.
    pub fn adjust_split(&mut self, new_kind: SplitKind) {
        if let Split(ref mut split) = self.ring.top {
            split.adjust_split(new_kind);
        }
    }

    /// Remove the split in the top panel of this section.
    pub fn unsplit(&mut self, save: SaveGrid) {
        if let Some(split) = self.ring.top.pull_split() {
            let ring = split.unsplit(save);
            if self.ring.len() == 1 {
                self.ring = ring;
            } else {
                self.ring.pop();
                self.ring.extend(ring.into_iter().rev());
            }
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
    pub fn is_fill(&self) -> bool {
        self.ring.top.is_fill()
    }

    // Count the number of visible grids in this section of the screen
    pub fn count_leaves(&self) -> usize {
        match self.ring.top {
            Fill(_)                             => 1,
            Split(ref split)                    => split.count_leaves(),
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
    pub fn fill(&self) -> &T {
        match self.ring.top {
            Fill(ref fill) => fill,
            _ => panic!("Cannot call fill on a split section of the screen"),
        }
    }

    /// Get a mutable reference to the grid associated with this section - panic if this section
    /// is split.
    pub fn fill_mut(&mut self) -> &mut T {
        match self.ring.top {
            Fill(ref mut fill) => fill,
            _ => panic!("Cannot call fill on a split section of the screen"),
        }
    }

    /// Get a reference to the children section of this section if it is split.
    pub fn children(&self) -> Option<(&ScreenSection<T>, &ScreenSection<T>)> {
        if let Split(ref split) = self.ring.top {
            Some(split.children())
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

    /// Iterate over all of the visible leaf panels in this section of the screen.
    pub fn panels(&self) -> super::Panels<T> {
        super::Panels {
            stack: vec![self],
        }
    }

}

impl<T: Resizeable> Resizeable for ScreenSection<T> {
    fn dims(&self) -> (u32, u32) {
        (self.area().width(), self.area().height())
    }
    
    fn resize_width(&mut self, width: u32) {
        let height = self.area.height();
        self.area = self.area.resized(width, height);
        for panel in &mut self.ring {
            panel.resize_width(width);
        }
    }

    fn resize_height(&mut self, height: u32) {
        let width = self.area.width();
        self.area = self.area.resized(width, height);
        for panel in &mut self.ring {
            panel.resize_height(height);
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.area = self.area.resized(width, height);
        for panel in &mut self.ring {
            panel.resize(width, height);
        }
    }
}

impl<T: Index<Coords>> Index<Coords> for ScreenSection<T> {
    type Output = T::Output;
    fn index(&self, coords: Coords) -> &Self::Output {
        match self.ring.top {
            Fill(ref fill)      => &fill[coords],
            Split(ref split)    => &split[coords],
            _ => unreachable!()
        }
    }
}

#[cfg(test)]
mod tests {
    pub use terminal::screen::tests::*;

    fn split(mut section: ScreenSection<MockFill>, save: SaveGrid) -> ScreenSection<MockFill> {
        section.split(save, Horizontal(4), Percentage, 3, 4, false);
        section
    }

    mod grid {
        use super::*;

        pub fn section() -> ScreenSection<MockFill> {
            ScreenSection::with_data(0, Region::new(0, 0, 8, 8), Fill(GRID))
        }

        #[test]
        fn split_save_left() {
            assert_eq!(super::split(section(), SaveGrid::Left), ScreenSection {
                tag: 0,
                area: Region::new(0, 0, 8, 8),
                ring: Ring::new(Split(SplitSection::new(
                    Box::new(ScreenSection::new(3, Region::new(0, 0, 8, 4), false)),
                    Box::new(ScreenSection::new(4, Region::new(0, 4, 8, 8), false)),
                    Region::new(0, 0, 8, 8),
                    Horizontal(4),
                )))
            })
        }

        #[test]
        fn split_save_right() {
            assert_eq!(super::split(section(), SaveGrid::Right), ScreenSection {
                tag: 0,
                area: Region::new(0, 0, 8, 8),
                ring: Ring::new(Split(SplitSection::new(
                    Box::new(ScreenSection::new(3, Region::new(0, 0, 8, 4), false)),
                    Box::new(ScreenSection::new(4, Region::new(0, 4, 8, 8), false)),
                    Region::new(0, 0, 8, 8),
                    Horizontal(4),
                )))
            })
        }

        #[test]
        fn is_fill() {
            assert!(section().is_fill())
        }

        #[test]
        fn count_leaves() {
            assert_eq!(section().count_leaves(), 1);
        }
    }

    mod split {
        use super::*;

        pub fn section() -> ScreenSection<MockFill> {
            ScreenSection {
                tag: 0,
                area: Region::new(0, 0, 8, 8),
                ring: Ring::new(Split(SplitSection::new(
                    Box::new(ScreenSection::new(1, Region::new(0, 0, 4, 8), false)),
                    Box::new(ScreenSection::new(2, Region::new(4, 0, 8, 8), false)),
                    Region::new(0, 0, 8, 8),
                    Vertical(4),
                ))),
            }
        }

        #[test]
        fn split_save_left() {
            assert_eq!(super::split(section(), SaveGrid::Left), ScreenSection {
                tag: 0,
                area: Region::new(0, 0, 8, 8),
                ring: Ring::new(Split(SplitSection::new(
                    Box::new(ScreenSection {
                        tag: 3,
                        area: Region::new(0, 0, 8, 4),
                        ring: Ring::new(Split(SplitSection::new(
                            Box::new(ScreenSection::new(1, Region::new(0, 0, 4, 4), false)),
                            Box::new(ScreenSection::new(2, Region::new(4, 0, 8, 4), false)),
                            Region::new(0, 0, 8, 4),
                            Vertical(4),
                        )))
                    }),
                    Box::new(ScreenSection::new(4, Region::new(0, 4, 8, 8), false)),
                    Region::new(0, 0, 8, 8),
                    Horizontal(4),
                )))
            });
        }

        #[test]
        fn split_save_right() {
            assert_eq!(super::split(section(), SaveGrid::Right), ScreenSection {
                tag: 0,
                area: Region::new(0, 0, 8, 8),
                ring: Ring::new(Split(SplitSection::new(
                    Box::new(ScreenSection::new(3, Region::new(0, 0, 8, 4), false)),
                    Box::new(ScreenSection {
                        tag: 4,
                        area: Region::new(0, 4, 8, 8),
                        ring: Ring::new(Split(SplitSection::new(
                            Box::new(ScreenSection::new(1, Region::new(0, 4, 4, 8), false)),
                            Box::new(ScreenSection::new(2, Region::new(4, 4, 8, 8), false)),
                            Region::new(0, 4, 8, 8),
                            Vertical(4),
                       ))),
                    }),
                    Region::new(0, 0, 8, 8),
                    Horizontal(4),
                ))),
            });
        }

        #[test]
        fn is_fill() {
            assert!(!section().is_fill())
        }

        #[test]
        fn count_leaves() {
            assert_eq!(section().count_leaves(), 2);
        }
    }

    mod ring {
        use super::*;

        pub fn section() -> ScreenSection<MockFill> {
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
                    let mut ring = Ring::new(Split(SplitSection::new(
                        Box::new(ScreenSection::new(1, Region::new(0, 0, 4, 8), false)),
                        Box::new(ScreenSection::new(2, Region::new(4, 0, 8, 8), false)),
                        Region::new(0, 0, 8, 8),
                        Vertical(4),
                    )));
                    ring.push(Split(SplitSection::new(
                        Box::new(ScreenSection::new(3, Region::new(0, 0, 8, 4), false)),
                        Box::new(ScreenSection::new(4, Region::new(0, 4, 8, 8), false)),
                        Region::new(0, 0, 8, 8),
                        Horizontal(4),
                    )));
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
                    let mut ring = Ring::new(Split(SplitSection::new(
                        Box::new(ScreenSection::new(1, Region::new(0, 0, 4, 8), false)),
                        Box::new(ScreenSection::new(2, Region::new(4, 0, 8, 8), false)),
                        Region::new(0, 0, 8, 8),
                        Vertical(4),
                    )));
                    ring.push(Split(SplitSection::new(
                        Box::new(ScreenSection::new(3, Region::new(0, 0, 8, 4), false)),
                        Box::new(ScreenSection::new(4, Region::new(0, 4, 8, 8), false)),
                        Region::new(0, 0, 8, 8),
                        Horizontal(4),
                    )));
                    ring
                },
            })
        }

        #[test]
        fn is_fill() {
            assert!(section().is_fill())
        }

        #[test]
        fn count_leaves() {
            assert_eq!(section().count_leaves(), 1);
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
            section.resize(6, 6);
            section
        }, [
            ScreenSection::new(0, Region::new(0, 0, 6, 6), false),
            ScreenSection {
                tag: 0,
                area: Region::new(0, 0, 6, 6),
                ring: Ring::new(Split(SplitSection::new(
                    Box::new(ScreenSection::new(1, Region::new(0, 0, 3, 6), false)),
                    Box::new(ScreenSection::new(2, Region::new(3, 0, 6, 6), false)),
                    Region::new(0, 0, 6, 6),
                    Vertical(3),
                ))),
            },
            ScreenSection {
                tag: 0,
                area: Region::new(0, 0, 6, 6),
                ring: {
                    let mut ring = Ring::new(Split(SplitSection::new(
                        Box::new(ScreenSection::new(1, Region::new(0, 0, 3, 6), false)),
                        Box::new(ScreenSection::new(2, Region::new(3, 0, 6, 6), false)),
                        Region::new(0, 0, 6, 6),
                        Vertical(3),
                    )));
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
                ring: Ring::new(Split(SplitSection::new(
                    Box::new(ScreenSection::new(1, Region::new(0, 0, 8, 4), false)),
                    Box::new(ScreenSection::new(2, Region::new(0, 4, 8, 8), false)),
                    Region::new(0, 0, 8, 8),
                    Horizontal(4),
                ))),
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
