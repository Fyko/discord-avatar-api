use http::Request;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tower_http::request_id::{MakeRequestId, RequestId};

// A `MakeRequestId` that increments an atomic counter
#[derive(Clone, Default)]
pub struct MyRequestId {
    counter: Arc<AtomicU64>,
}

impl MakeRequestId for MyRequestId {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let request_id = self
            .counter
            .fetch_add(1, Ordering::SeqCst)
            .to_string()
            .parse()
            .unwrap();

        Some(RequestId::new(request_id))
    }
}
