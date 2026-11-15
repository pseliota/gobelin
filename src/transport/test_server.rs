use std::{
    collections::HashMap,
    io::{ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use reqwest::Url;

pub(crate) enum Reply {
    Response {
        status: &'static str,
        headers: &'static str,
        body: String,
    },
    Close,
    Stall,
}

pub(crate) struct RecordedRequest {
    pub(crate) method: String,
    pub(crate) target: String,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: Vec<u8>,
}

impl RecordedRequest {
    pub(crate) fn path(&self) -> &str {
        self.target.split('?').next().unwrap()
    }

    pub(crate) fn query(&self) -> Vec<(String, String)> {
        Url::parse(&format!("http://localhost{}", self.target))
            .unwrap()
            .query_pairs()
            .into_owned()
            .collect()
    }

    pub(crate) fn param(&self, name: &str) -> Option<String> {
        self.query()
            .into_iter()
            .find_map(|(key, value)| (key == name).then_some(value))
    }
}

pub(crate) struct Recording(JoinHandle<Vec<RecordedRequest>>);

impl Recording {
    pub(crate) fn requests(self) -> Vec<RecordedRequest> {
        self.0.join().expect("test server failed")
    }

    pub(crate) fn request(self) -> RecordedRequest {
        let mut requests = self.requests();
        assert_eq!(requests.len(), 1, "expected exactly one request");
        requests.remove(0)
    }
}

pub(crate) fn serve(script: Vec<Reply>) -> (Url, Recording) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let address = listener.local_addr().unwrap();
    let handle = thread::spawn(move || {
        script
            .into_iter()
            .map(|reply| exchange(&listener, reply))
            .collect()
    });
    let base_url = Url::parse(&format!("http://{address}/")).unwrap();
    (base_url, Recording(handle))
}

fn exchange(listener: &TcpListener, reply: Reply) -> RecordedRequest {
    let mut stream = accept(listener);
    let mut bytes = Vec::new();
    let header_end = loop {
        let mut buffer = [0; 4096];
        let count = stream.read(&mut buffer).unwrap();
        assert!(count > 0, "incomplete headers");
        bytes.extend_from_slice(&buffer[..count]);
        if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
            break end + 4;
        }
        assert!(bytes.len() < 65536, "oversized headers");
    };
    let header_text = String::from_utf8(bytes[..header_end].to_vec()).unwrap();
    let mut lines = header_text.split("\r\n");
    let mut request_line = lines.next().unwrap().split_whitespace();
    let method = request_line.next().unwrap().to_owned();
    let target = request_line.next().unwrap().to_owned();
    let headers: HashMap<_, _> = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(key, value)| (key.to_ascii_lowercase(), value.trim().to_owned()))
        .collect();
    let length = headers
        .get("content-length")
        .map(|n| n.parse::<usize>().unwrap())
        .unwrap_or(0);
    while bytes.len() < header_end + length {
        let mut buffer = [0; 4096];
        let count = stream.read(&mut buffer).unwrap();
        assert!(count > 0, "incomplete body");
        bytes.extend_from_slice(&buffer[..count]);
    }
    match reply {
        Reply::Response {
            status,
            headers,
            body,
        } => {
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{headers}\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
        Reply::Close => {}
        Reply::Stall => while stream.read(&mut [0; 64]).is_ok_and(|count| count > 0) {},
    }
    RecordedRequest {
        method,
        target,
        headers,
        body: bytes[header_end..header_end + length].to_vec(),
    }
}

fn accept(listener: &TcpListener) -> TcpStream {
    let deadline = Instant::now() + Duration::from_secs(5);
    let stream = loop {
        match listener.accept() {
            Ok((stream, _)) => break stream,
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                assert!(Instant::now() < deadline, "client never connected");
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => panic!("accept failed: {error}"),
        }
    };
    stream.set_nonblocking(false).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream
}
