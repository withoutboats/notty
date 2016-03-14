use command::prelude::*;

use notty_encoding::cmds::{
    PushPanel, PopPanel,
    SplitPanel, UnsplitPanel,
    SwitchActivePanel,
};

impl Command for PushPanel {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.push(self.0);
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("PUSH BUFFER")
    }
}

impl Command for PopPanel {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.pop(self.0);
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("POP BUFFER")
    }
}

impl Command for SplitPanel {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.split(self.save, self.kind, self.rule, self.split_tag, self.l_tag, self.r_tag);
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("SPLIT BUFFER")
    }
}

impl Command for UnsplitPanel {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.unsplit(self.save, self.unsplit_tag);
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("UNSPLIT BUFFER")
    }
}

impl Command for SwitchActivePanel {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.switch(self.0);
        Ok(())
    }
    fn repr(&self) -> String {
        format!("SWITCH TO PANEL {}", self.0)
    }
}

