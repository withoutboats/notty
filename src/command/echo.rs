use notty_encoding::cmds::{SetBufferMode, SetEchoMode};

use command::prelude::*;

impl Command for SetBufferMode {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.set_buffer_mode(self.0);
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("SET BUFFER MODE")
    }
}

impl Command for SetEchoMode {
    fn apply(&self, terminal: &mut Terminal) -> io::Result<()> {
        terminal.set_echo_mode(self.0);
        Ok(())
    }
    fn repr(&self) -> String {
        String::from("SET ECHO MODE")
    }
}
