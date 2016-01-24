use std::mem;

use self::GridHierarchy::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SplitKind {
    Horizontal(u32),
    Vertical(u32),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum GridHierarchy {
    Grid(u64),
    Split {
        kind: SplitKind,
        left: Box<GridHierarchy>,
        right: Box<GridHierarchy>,
    },
}

impl GridHierarchy {
    pub fn remove(&mut self, tag: u64) -> bool {
        if let Some(new_grid) = if let Split { ref left, ref right, .. } = *self {
            match (left, right) {
                (&box Grid(ltag), _) if ltag == tag => Some(*right.clone()),
                (_, &box Grid(rtag)) if rtag == tag => Some(*left.clone()),
                _                                   => None
            }
        } else { None } {
            mem::replace(self, new_grid);
            true
        } else {
            if let Split { ref mut left, ref mut right, .. } = *self {
                if let Split { .. } = **left {
                    if left.remove(tag) { return true; }
                }
                if let Split { .. } = **right {
                    if right.remove(tag) { return true; }
                }
            }
            false
        } 
    }

    pub fn replace(&mut self, tag: u64, new: GridHierarchy) {
        self.find_mut(tag).map(|grid| *grid = new);
    }

    fn find_mut(&mut self, tag: u64) -> Option<&mut GridHierarchy> {
        match *self {
            Grid(id) if id == tag => Some(self),
            Split { ref mut left, ref mut right, .. } => {
                left.find_mut(tag).or_else(move || right.find_mut(tag))
            }
            _ => None
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use super::GridHierarchy::*;

    // The hierarchy this sets up is:
    //  _
    //  | \
    //  _  1
    //  | \
    //  0 0x0beefdad
    // Beef Dad is the needle for these tests.
    fn setup_grid_hierarchy() -> GridHierarchy {
        Split {
            kind: SplitKind::Horizontal(2),
            left: Box::new(Split {
                kind: SplitKind::Horizontal(2),
                left: Box::new(Grid(0)),
                right: Box::new(Grid(0x0beefdad)),
            }),
            right: Box::new(Grid(1)),
        }
    }

    // After this test:
    // _
    // | \
    // 0  1
    #[test]
    fn remove_a_tag() {
        let mut gh = setup_grid_hierarchy();
        gh.remove(0x0beefdad);
        assert_eq!(gh, Split {
            kind: SplitKind::Horizontal(2),
            left: Box::new(Grid(0)),
            right: Box::new(Grid(1)),
        })
    }

    // After this test:
    // _
    // | \
    // _  1
    // | \
    // 0 0x0badcafe
    #[test]
    fn replace_a_tag() {
        let mut gh = setup_grid_hierarchy();
        gh.replace(0x0beefdad, GridHierarchy::Grid(0x0badcafe));
        assert_eq!(gh, Split {
            kind: SplitKind::Horizontal(2),
            left: Box::new(Split {
                kind: SplitKind::Horizontal(2),
                left: Box::new(Grid(0)),
                right: Box::new(Grid(0x0badcafe)),
            }),
            right: Box::new(Grid(1)),
        })
    }
}
