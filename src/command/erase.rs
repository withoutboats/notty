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
use notty_encoding::cmds::{Erase, RemoveChars, RemoveRows, InsertBlank, InsertRows};

use command::prelude::*;

impl Command for Erase {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.erase(self.area);
        Ok(())
    }

    #[cfg(any(test, debug_assertions))]
    fn repr(&self) -> String {
        String::from("ERASE")
    }
}

impl Command for RemoveChars {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.remove_at(self.count);
        Ok(())
    }

    #[cfg(any(test, debug_assertions))]
    fn repr(&self) -> String {
        format!("REMOVE {} CHARS", self.count)
    }
}

impl Command for RemoveRows {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.remove_rows_at(self.count, self.include);
        Ok(())
    }

    #[cfg(any(test, debug_assertions))]
    fn repr(&self) -> String {
        match self.include {
            true    => format!("REMOVE {} ROWS INCL CURSOR", self.count),
            false   => format!("REMOVE {} ROWS BELOW CURSOR", self.count),
        }
    }
}

impl Command for InsertBlank {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.insert_blank_at(self.count);
        Ok(())
    }

    #[cfg(any(test, debug_assertions))]
    fn repr(&self) -> String {
        format!("INSERT {} BLANK SPACES", self.count)
    }
}

impl Command for InsertRows {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.insert_rows_at(self.count, self.include);
        Ok(())
    }

    #[cfg(any(test, debug_assertions))]
    fn repr(&self) -> String {
        match self.include {
            true    => format!("INSERT {} ROWS ABOVE CURSOR", self.count),
            false   => format!("INSERT {} ROWS BELOW CURSOR", self.count),
        }
    }
}
