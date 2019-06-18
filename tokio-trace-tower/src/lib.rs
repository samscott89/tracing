extern crate tower_layer;
extern crate tower_service;
#[macro_use]
extern crate tokio_trace as trace;
extern crate tokio_trace_futures as trace_futures;

#[macro_use]
extern crate futures;

use std::fmt;
use tower_service::Service;
use trace::{field, Level};
use trace_futures::{Instrument, Instrumented};

pub mod instrument;
pub mod request;

#[derive(Clone, Debug)]
pub struct InstrumentedService<S> {
    inner: S,
    span: trace::Span,
}

pub trait InstrumentableService<Request>: Service<Request> + Sized {
    fn instrument(self, span: tokio_trace::Span) -> InstrumentedService<Self> {
        InstrumentedService { inner: self, span }
    }
}

impl<T: Service<Request>, Request> Service<Request> for InstrumentedService<T>
where
    // TODO: it would be nice to do more for HTTP services...
    Request: fmt::Debug + Clone + Send + Sync + 'static,
{
    type Response = T::Response;
    type Error = T::Error;
    type Future = Instrumented<T::Future>;

    fn poll_ready(&mut self) -> futures::Poll<(), Self::Error> {
        let _enter = self.span.enter();
        self.inner.poll_ready()
    }

    fn call(&mut self, req: Request) -> Self::Future {
        // TODO: custom `Value` impls for `http` types would be nice...
        let span =
            span!(Level::TRACE, parent: &self.span, "request", request = &field::debug(&req));
        let enter = span.enter();
        self.inner.call(req).instrument(span.clone())
    }
}
