//! Attaching to a session: the frame-level client IVE would use, and an interactive attach for a terminal.

use std::io::{self, Write};
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use std::sync::mpsc;
use std::time::Duration;

use crate::host::poll;
use crate::protocol::{Frame, PROTOCOL, Size};
use crate::registry::{Registry, validate_id};

/// Ctrl-], as in telnet: detaches, leaving the program running.
pub const DETACH_KEY: u8 = 0x1d;
const SIZE_CHECK_INTERVAL_MS: i32 = 250;

/// How long the host may take to greet a new client.
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(3);

pub struct Client {
    stream: UnixStream,
    program_pid: u32,
}

impl Client {
    /// Attaches to the session, taking it over from any client attached before. The host sends nothing more until
    /// the first `Resize`.
    pub fn connect(registry: &Registry, id: &str) -> io::Result<Client> {
        validate_id(id)?;
        let stream = UnixStream::connect(registry.socket(id))
            .map_err(|error| io::Error::new(error.kind(), format!("cannot attach to session {id}: {error}")))?;
        Client::from_stream(stream).map_err(|error| io::Error::new(error.kind(), format!("session {id}: {error}")))
    }

    /// A client over an already connected stream; reads the host's handshake.
    pub fn from_stream(mut stream: UnixStream) -> io::Result<Client> {
        stream.set_read_timeout(Some(HANDSHAKE_TIMEOUT))?;
        let greeting = Frame::read_from(&mut stream)?;
        stream.set_read_timeout(None)?;
        match greeting {
            Frame::Attached { protocol: PROTOCOL, program_pid } => Ok(Client { stream, program_pid }),
            Frame::Attached { protocol, .. } => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("host speaks protocol {protocol}, this client speaks {PROTOCOL}"),
            )),
            other => Err(io::Error::new(io::ErrorKind::InvalidData, format!("expected the host's handshake, got {other:?}"))),
        }
    }

    /// The program the session runs, which leads its own process group and session.
    pub fn program_pid(&self) -> u32 {
        self.program_pid
    }

    /// Ends the program as closing its terminal would; the host reports the exit.
    pub fn hang_up(&mut self) -> io::Result<()> {
        self.send(&Frame::HangUp)
    }

    pub fn send(&mut self, frame: &Frame) -> io::Result<()> {
        frame.write_to(&mut self.stream)
    }

    pub fn receive(&mut self) -> io::Result<Frame> {
        Frame::read_from(&mut self.stream)
    }

    pub fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        self.stream.set_read_timeout(timeout)
    }

    /// Waits until a frame starts arriving, up to `timeout` (`None`: no limit); false when the time ran out. Unlike
    /// a read timeout, it never cuts a frame in half.
    pub fn wait_readable(&self, timeout: Option<Duration>) -> io::Result<bool> {
        let mut fds = [libc::pollfd { fd: self.stream.as_raw_fd(), events: libc::POLLIN, revents: 0 }];
        let millis = timeout.map_or(-1, |timeout| timeout.as_millis().min(i32::MAX as u128) as i32);
        poll(&mut fds, millis)?;
        Ok(fds[0].revents != 0)
    }

    /// Closes the connection for this client and all its clones; their reads end.
    pub fn shutdown(&self) {
        // fails only when the connection is already gone
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
    }

    pub fn try_clone(&self) -> io::Result<Client> {
        Ok(Client { stream: self.stream.try_clone()?, program_pid: self.program_pid })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ending {
    Detached,
    Exited(i32),
    /// the host went away or another client took over
    Lost,
}

/// Connects this process's terminal to the session until Ctrl-] detaches or the program exits.
pub fn attach_terminal(registry: &Registry, id: &str) -> io::Result<Ending> {
    let client = Client::connect(registry, id)?;
    let _raw = RawMode::enable()?;
    let (ended, ending) = mpsc::channel();
    let mut output = client.try_clone()?;
    std::thread::spawn(move || ended.send(pump_output(&mut output)));
    pump_input(client, &ending)
}

fn pump_output(client: &mut Client) -> Ending {
    let mut stdout = io::stdout().lock();
    loop {
        match client.receive() {
            Ok(Frame::Output(bytes) | Frame::Replay(bytes)) => {
                if stdout.write_all(&bytes).and_then(|()| stdout.flush()).is_err() {
                    return Ending::Lost;
                }
            }
            Ok(Frame::Exited(code)) => return Ending::Exited(code),
            Ok(_) => {}
            Err(_) => return Ending::Lost,
        }
    }
}

fn pump_input(mut client: Client, ending: &mpsc::Receiver<Ending>) -> io::Result<Ending> {
    let mut size = terminal_size();
    client.send(&Frame::Resize(size))?;
    let mut buffer = [0u8; 4096];
    loop {
        if let Ok(ended) = ending.try_recv() {
            return Ok(ended);
        }
        if terminal_size() != size {
            size = terminal_size();
            client.send(&Frame::Resize(size))?;
        }
        let read = read_stdin(&mut buffer)?;
        if let Some(detach) = buffer[..read].iter().position(|&b| b == DETACH_KEY) {
            client.send(&Frame::Input(buffer[..detach].to_vec()))?;
            return Ok(Ending::Detached);
        }
        if read > 0 {
            client.send(&Frame::Input(buffer[..read].to_vec()))?;
        }
    }
}

/// Reads what stdin has within the size check interval, unbuffered so poll sees everything pending.
fn read_stdin(buffer: &mut [u8]) -> io::Result<usize> {
    let mut fds = [libc::pollfd { fd: libc::STDIN_FILENO, events: libc::POLLIN, revents: 0 }];
    poll(&mut fds, SIZE_CHECK_INTERVAL_MS)?;
    if fds[0].revents == 0 {
        return Ok(0);
    }
    // SAFETY: reads into a valid buffer of the given length
    let read = unsafe { libc::read(libc::STDIN_FILENO, buffer.as_mut_ptr().cast(), buffer.len()) };
    match read {
        -1 => Err(io::Error::last_os_error()),
        0 => Err(io::Error::new(io::ErrorKind::UnexpectedEof, "stdin closed")),
        read => Ok(read as usize),
    }
}

pub fn terminal_size() -> Size {
    let mut window = libc::winsize { ws_row: 0, ws_col: 0, ws_xpixel: 0, ws_ypixel: 0 };
    // SAFETY: TIOCGWINSZ writes into a valid winsize
    let result = unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut window) };
    if result == -1 || window.ws_col == 0 || window.ws_row == 0 {
        return Size::DEFAULT;
    }
    Size { columns: window.ws_col, lines: window.ws_row }
}

/// Raw mode on stdin while attached, so keys like Ctrl-C reach the program; restored on drop.
struct RawMode {
    saved: libc::termios,
}

impl RawMode {
    fn enable() -> io::Result<RawMode> {
        // SAFETY: termios is plain data, filled by tcgetattr before use
        let mut saved: libc::termios = unsafe { std::mem::zeroed() };
        if unsafe { libc::tcgetattr(libc::STDIN_FILENO, &mut saved) } == -1 {
            return Err(io::Error::new(io::ErrorKind::Unsupported, "attach needs a terminal on stdin"));
        }
        let mut raw = saved;
        // SAFETY: valid termios pointers
        unsafe {
            libc::cfmakeraw(&mut raw);
            libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &raw);
        }
        Ok(RawMode { saved })
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        // SAFETY: restores what tcgetattr returned
        unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &self.saved) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair() -> (Client, UnixStream) {
        let (client_end, mut host_end) = UnixStream::pair().unwrap();
        Frame::Attached { protocol: PROTOCOL, program_pid: 7 }.write_to(&mut host_end).unwrap();
        (Client::from_stream(client_end).unwrap(), host_end)
    }

    #[test]
    fn should_wait_until_a_frame_is_readable() {
        // given
        let (client, mut host) = pair();
        let before = client.wait_readable(Some(Duration::from_millis(50))).unwrap();

        // when
        Frame::Output(b"x".to_vec()).write_to(&mut host).unwrap();
        let after = client.wait_readable(Some(Duration::from_millis(50))).unwrap();

        // then
        assert!(!before);
        assert!(after);
    }

    #[test]
    fn should_end_reads_of_clones_on_shutdown() {
        // given
        let (client, _host) = pair();
        let mut reader = client.try_clone().unwrap();

        // when
        client.shutdown();

        // then
        assert!(reader.receive().is_err());
    }
}
