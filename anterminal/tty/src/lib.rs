use std::io;
use std::ffi::CString;
use std::ptr;
use std::sync::Arc;

extern crate libc;

static TIOCSWINSZ: libc::c_ulong = 0x5414;

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

pub fn pty(name: &str, width: u16, height: u16) -> (Reader, Writer, Arc<Handle>) {
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
            (Reader(handle.clone()), Writer(handle.clone()), handle)
        }
        _           => panic!("Fork failed.")
    }
}

pub struct Reader(Arc<Handle>);

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

impl ::std::ops::Deref for Handle {
    type Target = libc::c_int;
    fn deref(&self) -> &libc::c_int { &self.0 }
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe { libc::close(self.0); }
    }
}
