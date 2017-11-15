use std::mem;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::io::{RawFd, AsRawFd};
use libc;
use cgmath::Vector2;
use error::{Error, Result};

struct WinSize {
    ws_row: libc::c_ushort,
    ws_col: libc::c_ushort,
    _ws_xpixel: libc::c_ushort,
    _ws_ypixel: libc::c_ushort
}

pub struct UnixBackend {
    tty_file: File,
    tty_fd: RawFd,
    original_termios: libc::termios,
}

impl UnixBackend {
    fn init_tty(fd: RawFd) -> Result<libc::termios> {
        let mut termios = unsafe { mem::uninitialized() };
        let res = unsafe { libc::tcgetattr(fd, &mut termios) };
        if res != 0 {
            return Err(Error::last_os_error());
        }

        let original_termios = termios.clone();

        termios.c_iflag &= !(libc::IGNBRK | libc::BRKINT | libc::PARMRK | libc::ISTRIP |
                             libc::INLCR | libc::IGNCR | libc::ICRNL |
                             libc::IXON);
        termios.c_oflag &= !libc::OPOST;
        termios.c_lflag &= !(libc::ECHO | libc::ECHONL | libc::ICANON | libc::ISIG | libc::IEXTEN);
        termios.c_cflag &= !(libc::CSIZE | libc::PARENB);
        termios.c_cflag |= libc::CS8;
        termios.c_cc[libc::VMIN] = 0;
        termios.c_cc[libc::VTIME] = 0;

        let res = unsafe { libc::tcsetattr(fd, libc::TCSAFLUSH, &termios) };
        if res != 0 {
            return Err(Error::last_os_error());
        }

        Ok(original_termios)
    }

    pub fn new() -> Result<Self> {
        let tty_file = OpenOptions::new()
            .write(true)
            .read(true)
            .open("/dev/tty")?;

        let tty_fd = tty_file.as_raw_fd();
        let original_termios = Self::init_tty(tty_fd)?;

        Ok(Self {
            tty_file,
            original_termios,
            tty_fd,
        })
    }

    pub fn size(&self) -> Result<Vector2<u16>> {
        let mut win_size = WinSize { ws_row: 0, ws_col: 0, _ws_xpixel: 0, _ws_ypixel: 0 };
        unsafe {
            libc::ioctl(self.tty_fd, libc::TIOCGWINSZ, &mut win_size);
        }

        if win_size.ws_row == 0 || win_size.ws_col == 0 {
            Err(Error::last_os_error())
        } else {
            Ok(Vector2::new(win_size.ws_col as u16, win_size.ws_row as u16))
        }
    }

    pub fn send(&mut self, data: &[u8]) -> io::Result<()> {
        self.tty_file.write_all(data)
    }

    fn reset(&mut self) -> Result<()> {
        let res = unsafe {
            libc::tcsetattr(self.tty_fd, libc::TCSAFLUSH, &self.original_termios)
        };

        if res != 0 {
            return Err(Error::last_os_error());
        }

        Ok(())
    }
}

impl Drop for UnixBackend {
    fn drop(&mut self) {
        self.reset().expect("Failed to reset terminal to original settings");
    }
}
