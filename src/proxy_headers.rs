//! Middleware for recording proxy headers as span attributes.
//!
//! Extracts X-Forwarded-Proto and X-Forwarded-Port headers from reverse proxies
//! and records them to the current tracing span for proper OpenTelemetry semantics.

use axum::{extract::Request, response::Response};
use std::task::{Context, Poll};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct RecordProxyHeadersLayer;

impl<S> Layer<S> for RecordProxyHeadersLayer {
    type Service = RecordProxyHeadersService<S>;

    fn layer(&self, service: S) -> Self::Service {
        RecordProxyHeadersService { inner: service }
    }
}

#[derive(Clone)]
pub struct RecordProxyHeadersService<S> {
    inner: S,
}

impl<S, B> Service<Request<B>> for RecordProxyHeadersService<S>
where
    S: Service<Request<B>, Response = Response> + Clone + Send + 'static,
    S::Future: Send,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let scheme = req
            .headers()
            .get("x-forwarded-proto")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_owned());

        let port = req
            .headers()
            .get("x-forwarded-port")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<i64>().ok());

        if let Some(scheme) = scheme {
            tracing::Span::current().record("url.scheme", &scheme);
        }

        if let Some(port) = port {
            tracing::Span::current().record("server.port", port);
        }

        self.inner.call(req)
    }
}
