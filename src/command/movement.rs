use command::prelude::*;
use datatypes::Movement;
use datatypes::Movement::*;

#[derive(Copy, Clone)]
pub struct Move {
    movement: Movement,
}

impl Move {
    pub fn new(movement: Movement) -> Move {
        Move {
            movement: movement,
        }
    }
}

impl Command for Move {
    fn apply(&self, screen: &mut Screen, _: &Sender<InputEvent>) {
        screen.move_cursor(self.movement);
    }
    fn repr(&self) -> String {
        match self.movement {
            Up(n)               => format!("MOVE UP {}", n),
            Down(n)             => format!("MOVE DOWN {}", n),
            Left(n)             => format!("MOVE LEFT {}", n),
            Right(n)            => format!("MOVE RIGHT {}", n),
            PreviousLine(n)     => format!("MOVE PREV LINE {}", n),
            NextLine(n)         => format!("MOVE NEXT LINE {}", n),
            LeftTab(n)          => format!("MOVE LEFT TAB {}", n),
            RightTab(n)         => format!("MOVE RIGHT TAB {}", n),
            UpIndex(n)          => format!("MOVE UP INDEX {}", n),
            DownIndex(n)        => format!("MOVE DOWN INDEX {}", n),
            LeftIndex(n)        => format!("MOVE LEFT INDEX {}", n),
            RightIndex(n)       => format!("MOVE RIGHT INDEX {}", n),
            Column(n)           => format!("MOVE TO COL {}", n),
            Row(n)              => format!("MOVE TO ROW {}", n),
            Position(coords)    => format!("MOVE TO {},{}", coords.x, coords.y),
            UpToEdge            => String::from("MOVE UP TO EDGE"),
            DownToEdge          => String::from("MOVE DOWN TO EDGE"),
            LeftToEdge          => String::from("MOVE LEFT TO EDGE"),
            RightToEdge         => String::from("MOVE RIGHT TO EDGE"),
            ToBeginning         => String::from("MOVE TO BEGINNING"),
            ToEnd               => String::from("MOVE TO END"),
        }
    }
}

pub struct ScrollScreen {
    movement: Movement,
}

impl ScrollScreen {
    pub fn new(movement: Movement) -> ScrollScreen {
        ScrollScreen {
            movement: movement,
        }
    }
}

impl Command for ScrollScreen {
    fn apply(&self, screen: &mut Screen, _: &Sender<InputEvent>) {
        screen.scroll(self.movement)
    }
    fn repr(&self) -> String {
        String::from("SCROLL SCREEN")
    }
}
