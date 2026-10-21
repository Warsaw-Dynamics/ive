//! HTTP/1.1 over hyper for IVE's two servers, MCP and `ive --serve`: a task per connection, a handler per request.

use std::convert::Infallible;
use std::future::Future;
use std::time::Duration;

use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::header::{HeaderName, HeaderValue};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

pub type Body = BoxBody<Bytes, Infallible>;
pub type HttpResponse = Response<Body>;

/// How long to wait before accepting again when accepting failed (e.g. out of file descriptors).
const ACCEPT_RETRY: Duration = Duration::from_millis(100);

/// Answers every connection to `listener` with `handler`, until the runtime ends; `name` heads its log lines.
pub async fn serve<H, F>(listener: TcpListener, name: &'static str, handler: H)
where
    H: Fn(Request<Incoming>) -> F + Clone + Send + Sync + 'static,
    F: Future<Output = HttpResponse> + Send + 'static,
{
    loop {
        let stream = match listener.accept().await {
            Ok((stream, _)) => stream,
            Err(error) => {
                eprintln!("{name}: cannot accept a connection: {error}");
                tokio::time::sleep(ACCEPT_RETRY).await;
                continue;
            }
        };
        let handler = handler.clone();
        let service = service_fn(move |request| {
            let answered = handler(request);
            async move { Ok::<_, Infallible>(answered.await) }
        });
        // a connection that fails (the client went away mid-request) ends alone
        tokio::spawn(http1::Builder::new().serve_connection(TokioIo::new(stream), service));
    }
}

/// `body` with `status` and `headers`; a header that is not valid HTTP is left out.
pub fn respond(status: StatusCode, headers: &[(HeaderName, &str)], body: impl Into<Bytes>) -> HttpResponse {
    let mut response = Response::new(full(body));
    *response.status_mut() = status;
    for (name, value) in headers {
        if let Ok(value) = HeaderValue::try_from(*value) {
            response.headers_mut().insert(name.clone(), value);
        }
    }
    response
}

/// A plain-text answer, e.g. a 404.
pub fn text(status: StatusCode, text: &'static str) -> HttpResponse {
    respond(status, &[(hyper::header::CONTENT_TYPE, "text/plain; charset=utf-8")], text)
}

pub fn full(body: impl Into<Bytes>) -> Body {
    Full::new(body.into()).boxed()
}
