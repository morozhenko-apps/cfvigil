//! Deterministic fake Cloudflare transport over 127.0.0.1 only.
//! Exercising the REAL reqwest request stack tests auth-independent JSON,
//! HTTP failure, pagination and retry handling without any external calls.
//! This module is compiled exclusively under cfg(test).
use serde_json::Value;
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

#[derive(Clone, Debug)]
pub struct Observed {
    pub method: String,
    pub path: String,
    pub body: Value,
}
pub struct FixtureHttp {
    address: SocketAddr,
    seen: Arc<Mutex<Vec<Observed>>>,
    worker: Option<thread::JoinHandle<()>>,
}
impl FixtureHttp {
    pub fn start(
        route: impl Fn(&Observed) -> (u16, Value) + Send + 'static,
    ) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let observed = Arc::clone(&seen);
        let worker = thread::spawn(move || {
            for incoming in listener.incoming() {
                let mut stream = incoming.unwrap();
                stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                let mut bytes = Vec::new();
                let mut buf = [0u8; 2048];
                let header_end = loop {
                    let n = stream.read(&mut buf).unwrap_or(0);
                    if n == 0 { break None; }
                    bytes.extend_from_slice(&buf[..n]);
                    if let Some(offset) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                        break Some(offset + 4);
                    }
                    if bytes.len() > 128 * 1024 { break None; }
                };
                let Some(end) = header_end else { continue };
                let header = String::from_utf8_lossy(&bytes[..end]);
                let mut request_line = header.lines().next().unwrap_or("").split_whitespace();
                let method = request_line.next().unwrap_or("").to_owned();
                let path = request_line.next().unwrap_or("").to_owned();
                if path == "/__close__" { break; }
                let length:usize = header.lines().filter_map(|line| {
                    let (name,value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok()).flatten()
                }).next().unwrap_or(0);
                assert!(length <= 128*1024,"fixture payload over limit");
                while bytes.len() < end + length {
                    let n = stream.read(&mut buf).unwrap_or(0);
                    assert!(n > 0,"fixture client body cut short");
                    bytes.extend_from_slice(&buf[..n]);
                }
                let body = if length == 0 { Value::Null } else {
                    serde_json::from_slice(&bytes[end..end+length])
                        .expect("fixture request must be valid JSON")
                };
                let entry = Observed {method,path,body};
                observed.lock().unwrap().push(entry.clone());
                let (status,json) = route(&entry);
                let body = json.to_string();
                let reason = match status {
                    200=>"OK",400=>"Bad Request",401=>"Unauthorized",
                    403=>"Forbidden",429=>"Too Many Requests",
                    500=>"Internal Server Error", _=>"Unexpected",
                };
                let retry = if status==429 {"Retry-After: 75\r\n"} else {""};
                write!(stream,"HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: {}\r\n{retry}\r\n{}",body.len(),body).unwrap();
                stream.flush().unwrap();
            }
        });
        Self {address,seen,worker:Some(worker)}
    }
    pub fn url(&self,path:&str)->String { format!("http://{}{}",self.address,path) }
    pub fn requests(&self)->Vec<Observed> { self.seen.lock().unwrap().clone() }
}
impl Drop for FixtureHttp {
    fn drop(&mut self) {
        if let Ok(mut close) = TcpStream::connect(self.address) {
            let _ = close.write_all(b"GET /__close__ HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n");
        }
        if let Some(worker)=self.worker.take() { worker.join().unwrap(); }
    }
}
