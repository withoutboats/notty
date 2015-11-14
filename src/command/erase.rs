use command::prelude::*;
use datatypes::Area;

pub struct Erase {
    area: Area,
}

impl Erase {
    pub fn new(area: Area) -> Erase {
        Erase {
            area: area,
        }
    }
}

impl Command for Erase {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.erase(self.area);
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("ERASE")
    }
}

pub struct RemoveChars {
    count: u32,
}

impl RemoveChars {
    pub fn new(count: u32) -> RemoveChars {
        RemoveChars {
            count: count,
        }
    }
}

impl Command for RemoveChars {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.remove_at(self.count);
        Ok(())
    }
    fn repr(&self) -> String {
        format!("REMOVE {} CHARS", self.count)
    }
}

pub struct RemoveRows {
    count: u32,
    include: bool,
}

impl RemoveRows {
    pub fn new(count: u32, include_cu_row: bool) -> RemoveRows {
        RemoveRows {
            count: count,
            include: include_cu_row,
        }
    }
}

impl Command for RemoveRows {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.remove_rows_at(self.count, self.include);
        Ok(())
    }
    fn repr(&self) -> String {
        match self.include {
            true    => format!("REMOVE {} ROWS INCL CURSOR", self.count),
            false   => format!("REMOVE {} ROWS BELOW CURSOR", self.count),
        }
    }
}

pub struct InsertBlank {
    count: u32
}

impl InsertBlank {
    pub fn new(count: u32) -> InsertBlank {
        InsertBlank {
            count: count,
        }
    }
}

impl Command for InsertBlank {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.insert_blank_at(self.count);
        Ok(())
    }
    fn repr(&self) -> String {
        format!("INSERT {} BLANK SPACES", self.count)
    }
}

pub struct InsertRows {
    count: u32,
    include: bool
}

impl InsertRows {
    pub fn new(count: u32, include_cu_row: bool) -> InsertRows {
        InsertRows {
            count: count,
            include: include_cu_row,
        }
    }
}

impl Command for InsertRows {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.insert_rows_at(self.count, self.include);
        Ok(())
    }
    fn repr(&self) -> String {
        match self.include {
            true    => format!("INSERT {} ROWS ABOVE CURSOR", self.count),
            false   => format!("INSERT {} ROWS BELOW CURSOR", self.count),
        }
    }
}
