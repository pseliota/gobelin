use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use super::RecordedRequest;
use crate::transport::http::{
    Headers, HttpExecutor, HttpMethod, HttpRequest, HttpResponse, TransportFailure,
};

pub(crate) type Reply = Result<HttpResponse, TransportFailure>;

pub(crate) fn response(status: u16, headers: &[(&str, &str)], body: &str) -> Reply {
    let mut owned = Headers::default();
    for (name, value) in headers {
        owned.insert(name, value);
    }
    Ok(HttpResponse {
        status,
        headers: owned,
        body: body.as_bytes().to_vec(),
    })
}

pub(crate) fn interrupted() -> Reply {
    Err(TransportFailure::Interrupted { timed_out: false })
}

pub(crate) fn timed_out() -> Reply {
    Err(TransportFailure::Interrupted { timed_out: true })
}

pub(crate) fn executor(script: Vec<Reply>) -> (Box<dyn HttpExecutor>, Recording) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let executor = ScriptedExecutor {
        script: Mutex::new(script.into()),
        requests: Arc::clone(&requests),
    };
    (Box::new(executor), Recording(requests))
}

struct ScriptedExecutor {
    script: Mutex<VecDeque<Reply>>,
    requests: Arc<Mutex<Vec<HttpRequest>>>,
}

impl HttpExecutor for ScriptedExecutor {
    fn execute(&self, request: HttpRequest) -> Result<HttpResponse, TransportFailure> {
        self.requests.lock().unwrap().push(request);
        self.script
            .lock()
            .unwrap()
            .pop_front()
            .expect("request sent after the last scripted reply")
    }
}

pub(crate) struct Recording(Arc<Mutex<Vec<HttpRequest>>>);

impl Recording {
    pub(crate) fn sent(&self) -> Vec<HttpRequest> {
        self.0.lock().unwrap().clone()
    }

    pub(crate) fn requests(self) -> Vec<RecordedRequest> {
        self.sent().into_iter().map(recorded).collect()
    }

    pub(crate) fn request(self) -> RecordedRequest {
        let mut requests = self.requests();
        assert_eq!(requests.len(), 1, "expected exactly one request");
        requests.remove(0)
    }
}

fn recorded(request: HttpRequest) -> RecordedRequest {
    let target = match request.url.query() {
        Some(query) => format!("{}?{query}", request.url.path()),
        None => request.url.path().to_owned(),
    };
    RecordedRequest {
        method: verb(request.method).to_owned(),
        target,
        headers: request
            .headers
            .iter()
            .map(|header| (header.name().to_owned(), header.value().to_owned()))
            .collect(),
        body: request.body.unwrap_or_default(),
    }
}

fn verb(method: HttpMethod) -> &'static str {
    match method {
        HttpMethod::Get => "GET",
        HttpMethod::Post => "POST",
        HttpMethod::Patch => "PATCH",
        HttpMethod::Delete => "DELETE",
    }
}
