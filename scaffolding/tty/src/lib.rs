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
use std::io;
use std::ffi::CString;
use std::ops::Deref;
use std::ptr;
use std::sync::Arc;

extern crate libc;
extern crate notty;

use notty::terminal::Tty;

#[cfg(target_os="linux")]
const TIOCSWINSZ: libc::c_ulong = 0x5414;
#[cfg(target_os="macos")]
const TIOCSWINSZ: libc::c_ulong = 2148037735;

#[link(name="util")]
extern {
    fn forkpty(amaster: *mut libc::c_int,
               name: *mut libc::c_char,
               termp: *const libc::c_void,
               winsize: *const Winsize) -> libc::pid_t;
}

#[repr(C)]
struct Winsize {
    ws_row: libc::c_ushort,
    ws_col: libc::c_ushort,
    ws_xpixel: libc::c_ushort,
    ws_ypixel: libc::c_ushort,
}

pub fn pty(name: &str, width: u16, height: u16) -> (Reader, Writer) {
    let mut amaster = 0;
    let winsize = Winsize {
        ws_row: height as libc::c_ushort,
        ws_col: width as libc::c_ushort,
        ws_xpixel: 0,
        ws_ypixel: 0
    };
    match unsafe { forkpty(&mut amaster as *mut _,
                           ptr::null_mut(),
                           ptr::null(),
                           &winsize as *const _) } {
        0           => {
            let name = CString::new(name).unwrap();
            unsafe { libc::execvp(name.as_ptr(), ptr::null()); }
            unreachable!();
        }
        n if n > 0  => {
            let handle = Arc::new(Handle(amaster));
            (Reader(handle.clone()), Writer(handle.clone()))
        }
        _           => panic!("Fork failed.")
    }
}

pub struct Reader(Arc<Handle>);

impl Deref for Reader {
    type Target = Handle;
    fn deref(&self) -> &Handle {
        &*self.0
    }
}

impl io::Read for Reader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match unsafe {libc::read(**self.0, buf.as_mut_ptr() as *mut _, buf.len() as libc::size_t)}
        {
            n if n >= 0 => Ok(n as usize),
            _           => Err(io::Error::last_os_error()),
        }
    }
}

pub struct Writer(Arc<Handle>);

impl Deref for Writer {
    type Target = Handle;
    fn deref(&self) -> &Handle {
        &*self.0
    }
}

impl io::Write for Writer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match unsafe {libc::write(**self.0, buf.as_ptr() as *const _, buf.len() as libc::size_t)}
        {
            n if n >= 0 => Ok(n as usize),
            _           => Err(io::Error::last_os_error()),
        }
    }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

impl Tty for Writer {
    fn set_winsize(&mut self, width: u16, height: u16) -> io::Result<()> {
        (**self).set_winsize(width, height)
    }
}

pub struct Handle(libc::c_int);

impl Handle {
    pub fn set_winsize(&self, width: u16, height: u16) -> io::Result<()> {
        let winsize = Winsize {
            ws_row: height as libc::c_ushort,
            ws_col: width as libc::c_ushort,
            ws_xpixel: 0,
            ws_ypixel: 0
        };
        match unsafe { libc::ioctl(**self, TIOCSWINSZ, &winsize as *const _) } {
            -1  => Err(io::Error::last_os_error()),
            _   => Ok(()),
        }
    }
}

impl Deref for Handle {
    type Target = libc::c_int;
    fn deref(&self) -> &libc::c_int { &self.0 }
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe { libc::close(self.0); }
    }
}
