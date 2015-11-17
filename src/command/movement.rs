//  notty is a new kind of terminal emulator.
//  Copyright (C) 2015 without boats
//  
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Affero General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//  
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU Affero General Public License for more details.
//  
//  You should have received a copy of the GNU Affero General Public License
//  along with this program.  If not, see <http://www.gnu.org/licenses/>.
use notty_encoding::cmds::{Move, ScrollScreen};

use command::prelude::*;
use datatypes::Direction::*;
use datatypes::Movement::*;

impl Command for Move {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.move_cursor(self.movement);
        Ok(())
    }
    fn repr(&self) -> String {
        match self.movement {
            To(Up, n, _)        => format!("MOVE UP {}", n),
            To(Down, n, _)      => format!("MOVE DOWN {}", n),
            To(Left, n, _)      => format!("MOVE LEFT {}", n),
            To(Right, n, _)     => format!("MOVE RIGHT {}", n),
            PreviousLine(n)     => format!("MOVE PREV LINE {}", n),
            NextLine(n)         => format!("MOVE NEXT LINE {}", n),
            Tab(Up, n, _)       => format!("MOVE UP TAB {}", n),
            Tab(Down, n, _)     => format!("MOVE DOWN TAB {}", n),
            Tab(Left, n, _)     => format!("MOVE LEFT TAB {}", n),
            Tab(Right, n, _)    => format!("MOVE RIGHT TAB {}", n),
            IndexTo(Up, n)      => format!("MOVE UP INDEX {}", n),
            IndexTo(Down, n)    => format!("MOVE DOWN INDEX {}", n),
            IndexTo(Left, n)    => format!("MOVE LEFT INDEX {}", n),
            IndexTo(Right, n)   => format!("MOVE RIGHT INDEX {}", n),
            Column(n)           => format!("MOVE TO COL {}", n),
            Row(n)              => format!("MOVE TO ROW {}", n),
            Position(coords)    => format!("MOVE TO {},{}", coords.x, coords.y),
            ToEdge(Up)          => String::from("MOVE UP TO EDGE"),
            ToEdge(Down)        => String::from("MOVE DOWN TO EDGE"),
            ToEdge(Left)        => String::from("MOVE LEFT TO EDGE"),
            ToEdge(Right)       => String::from("MOVE RIGHT TO EDGE"),
            ToBeginning         => String::from("MOVE TO BEGINNING"),
            ToEnd               => String::from("MOVE TO END"),
        }
    }
}

impl Command for ScrollScreen {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.scroll(self.dir, self.n);
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("SCROLL SCREEN")
    }
}
