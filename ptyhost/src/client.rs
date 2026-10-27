//! Attaching to a session: the frame-level client IVE would use, and an interactive attach for a terminal.

use std::io::{self, Write};
use std::sync::mpsc;
use std::time::Duration;

use crate::net::UnixStream;
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
        let millis = timeout.map_or(-1, |timeout| timeout.as_millis().min(i32::MAX as u128) as i32);
        os::wait_readable(&self.stream, millis)
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
    let _raw = os::RawMode::enable()?;
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
        let read = os::read_stdin(&mut buffer)?;
        if let Some(detach) = buffer[..read].iter().position(|&b| b == DETACH_KEY) {
            client.send(&Frame::Input(buffer[..detach].to_vec()))?;
            return Ok(Ending::Detached);
        }
        if read > 0 {
            client.send(&Frame::Input(buffer[..read].to_vec()))?;
        }
    }
}

pub fn terminal_size() -> Size {
    os::terminal_size().filter(|size| size.columns > 0 && size.lines > 0).unwrap_or(Size::DEFAULT)
}

#[cfg(unix)]
use unix as os;
#[cfg(windows)]
use windows as os;

#[cfg(unix)]
mod unix {
    use std::io;
    use std::os::fd::AsRawFd;

    use super::SIZE_CHECK_INTERVAL_MS;
    use crate::host::poll;
    use crate::net::UnixStream;
    use crate::protocol::Size;

    pub fn wait_readable(stream: &UnixStream, millis: i32) -> io::Result<bool> {
        let mut fds = [libc::pollfd { fd: stream.as_raw_fd(), events: libc::POLLIN, revents: 0 }];
        poll(&mut fds, millis)?;
        Ok(fds[0].revents != 0)
    }

    /// Reads what stdin has within the size check interval, unbuffered so poll sees everything pending.
    pub fn read_stdin(buffer: &mut [u8]) -> io::Result<usize> {
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

    pub fn terminal_size() -> Option<Size> {
        let mut window = libc::winsize { ws_row: 0, ws_col: 0, ws_xpixel: 0, ws_ypixel: 0 };
        // SAFETY: TIOCGWINSZ writes into a valid winsize
        let result = unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut window) };
        (result != -1).then_some(Size { columns: window.ws_col, lines: window.ws_row })
    }

    /// Raw mode on stdin while attached, so keys like Ctrl-C reach the program; restored on drop.
    pub struct RawMode {
        saved: libc::termios,
    }

    impl RawMode {
        pub fn enable() -> io::Result<RawMode> {
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
}

#[cfg(windows)]
mod windows {
    use std::io::{self, Read};
    use std::os::windows::io::AsRawSocket;
    use std::sync::{Mutex, OnceLock, mpsc};
    use std::time::Duration;

    use windows_sys::Win32::Networking::WinSock::{POLLRDNORM, SOCKET, SOCKET_ERROR, WSAPOLLFD, WSAPoll};
    use windows_sys::Win32::System::Console::{
        CONSOLE_MODE, CONSOLE_SCREEN_BUFFER_INFO, DISABLE_NEWLINE_AUTO_RETURN, ENABLE_PROCESSED_OUTPUT,
        ENABLE_VIRTUAL_TERMINAL_INPUT, ENABLE_VIRTUAL_TERMINAL_PROCESSING, GetConsoleMode, GetConsoleScreenBufferInfo,
        GetStdHandle, STD_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, SetConsoleMode,
    };

    use super::SIZE_CHECK_INTERVAL_MS;
    use crate::net::UnixStream;
    use crate::protocol::Size;

    /// A read of stdin is never longer than this, nor than the buffer `pump_input` reads into.
    const READ_CHUNK: usize = 4096;

    pub fn wait_readable(stream: &UnixStream, millis: i32) -> io::Result<bool> {
        let mut fds = [WSAPOLLFD { fd: stream.as_raw_socket() as SOCKET, events: POLLRDNORM, revents: 0 }];
        // SAFETY: one valid WSAPOLLFD
        if unsafe { WSAPoll(fds.as_mut_ptr(), 1, millis) } == SOCKET_ERROR {
            return Err(io::Error::last_os_error());
        }
        Ok(fds[0].revents != 0)
    }

    /// Reads what stdin has within the size check interval. A console read cannot time out, so one thread reads
    /// stdin for the rest of the process and hands its reads over.
    pub fn read_stdin(buffer: &mut [u8]) -> io::Result<usize> {
        static READS: OnceLock<Mutex<mpsc::Receiver<io::Result<Vec<u8>>>>> = OnceLock::new();
        let reads = READS.get_or_init(|| Mutex::new(spawn_stdin_reader()));
        let reads = reads.lock().map_err(|_| io::Error::other("the stdin reader panicked"))?;
        match reads.recv_timeout(Duration::from_millis(SIZE_CHECK_INTERVAL_MS as u64)) {
            Ok(Ok(bytes)) if bytes.is_empty() => Err(io::Error::new(io::ErrorKind::UnexpectedEof, "stdin closed")),
            Ok(Ok(bytes)) => {
                buffer[..bytes.len()].copy_from_slice(&bytes);
                Ok(bytes.len())
            }
            Ok(Err(error)) => Err(error),
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(0),
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(io::Error::new(io::ErrorKind::UnexpectedEof, "stdin closed")),
        }
    }

    fn spawn_stdin_reader() -> mpsc::Receiver<io::Result<Vec<u8>>> {
        let (sender, reads) = mpsc::channel();
        std::thread::spawn(move || {
            let mut chunk = [0u8; READ_CHUNK];
            loop {
                let read = io::stdin().lock().read(&mut chunk).map(|read| chunk[..read].to_vec());
                let ended = !matches!(&read, Ok(bytes) if !bytes.is_empty());
                if sender.send(read).is_err() || ended {
                    return;
                }
            }
        });
        reads
    }

    pub fn terminal_size() -> Option<Size> {
        // SAFETY: plain data, filled by the call before use
        let mut info: CONSOLE_SCREEN_BUFFER_INFO = unsafe { std::mem::zeroed() };
        // SAFETY: a valid out-pointer; the handle is the process's own
        if unsafe { GetConsoleScreenBufferInfo(GetStdHandle(STD_OUTPUT_HANDLE), &mut info) } == 0 {
            return None;
        }
        let window = info.srWindow;
        Some(Size { columns: (window.Right - window.Left + 1) as u16, lines: (window.Bottom - window.Top + 1) as u16 })
    }

    /// The console in VT mode while attached: keys arrive as the bytes a terminal sends (Ctrl+C among them), and
    /// escape sequences in the output draw; both modes restored on drop.
    pub struct RawMode {
        input: CONSOLE_MODE,
        output: CONSOLE_MODE,
    }

    impl RawMode {
        pub fn enable() -> io::Result<RawMode> {
            let (Some(input), Some(output)) = (mode(STD_INPUT_HANDLE), mode(STD_OUTPUT_HANDLE)) else {
                return Err(io::Error::new(io::ErrorKind::Unsupported, "attach needs a console on stdin and stdout"));
            };
            set_mode(STD_INPUT_HANDLE, ENABLE_VIRTUAL_TERMINAL_INPUT);
            set_mode(STD_OUTPUT_HANDLE, ENABLE_PROCESSED_OUTPUT | ENABLE_VIRTUAL_TERMINAL_PROCESSING | DISABLE_NEWLINE_AUTO_RETURN);
            Ok(RawMode { input, output })
        }
    }

    impl Drop for RawMode {
        fn drop(&mut self) {
            set_mode(STD_INPUT_HANDLE, self.input);
            set_mode(STD_OUTPUT_HANDLE, self.output);
        }
    }

    fn mode(which: STD_HANDLE) -> Option<CONSOLE_MODE> {
        let mut mode = 0;
        // SAFETY: a valid out-pointer; the handle is the process's own
        (unsafe { GetConsoleMode(GetStdHandle(which), &mut mode) } != 0).then_some(mode)
    }

    fn set_mode(which: STD_HANDLE, mode: CONSOLE_MODE) {
        // SAFETY: the handle is the process's own
        unsafe { SetConsoleMode(GetStdHandle(which), mode) };
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
