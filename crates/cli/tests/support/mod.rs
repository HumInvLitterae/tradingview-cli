use assert_cmd::{Command, assert::Assert};
use serde_json::Value;
use std::io::{ErrorKind, Read};
use std::net::TcpListener;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

pub fn tv() -> Command {
    Command::cargo_bin("tv").expect("tv binary should build")
}

pub struct CdpDisconnectCommand {
    command: Command,
    #[allow(dead_code)]
    port: u16,
    stop: Arc<AtomicBool>,
    server: Option<JoinHandle<()>>,
}

impl CdpDisconnectCommand {
    #[allow(dead_code)]
    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Deref for CdpDisconnectCommand {
    type Target = Command;

    fn deref(&self) -> &Self::Target {
        &self.command
    }
}

impl DerefMut for CdpDisconnectCommand {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.command
    }
}

impl Drop for CdpDisconnectCommand {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.server
            .take()
            .expect("fixture server should be present")
            .join()
            .expect("join transport fixture");
    }
}

#[allow(dead_code)]
pub fn tv_with_cdp_disconnect() -> CdpDisconnectCommand {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind transport fixture");
    let port = listener.local_addr().expect("read fixture address").port();
    listener
        .set_nonblocking(true)
        .expect("make fixture listener nonblocking");
    let stop = Arc::new(AtomicBool::new(false));
    let server_stop = Arc::clone(&stop);
    let server = std::thread::spawn(move || {
        while !server_stop.load(Ordering::Acquire) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    stream
                        .set_nonblocking(false)
                        .expect("make fixture stream blocking");
                    stream
                        .set_read_timeout(Some(Duration::from_secs(5)))
                        .expect("set fixture read timeout");
                    let mut request = Vec::new();
                    let mut chunk = [0_u8; 1024];
                    while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                        let read = stream.read(&mut chunk).expect("read CLI request");
                        if read == 0 {
                            break;
                        }
                        request.extend_from_slice(&chunk[..read]);
                    }
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("accept CLI request: {error}"),
            }
        }
    });
    let mut command = tv();
    command.env("TV_CDP_HOST", "127.0.0.1");
    command.env("TV_CDP_PORT", port.to_string());
    CdpDisconnectCommand {
        command,
        port,
        stop,
        server: Some(server),
    }
}

pub fn stderr_json(assert: &Assert) -> Value {
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    serde_json::from_str(&stderr).unwrap()
}
