//! A generic widget.
//!
//! [`Widget`] is a useful generic container for UI and a stream of events.

use std::pin::Pin;

use futures_lite::{Stream, StreamExt};
use mogwai::prelude::*;

/// Container for some arbitrary user interface paired with an event stream.
#[derive(ViewChild)]
pub struct Widget<V: View, T> {
    #[child]
    wrapper: V::Element,
    stream: Pin<Box<dyn Stream<Item = T>>>,
}

impl<V: View, T> Widget<V, T> {
    pub fn new(wrapper: V::Element, stream: impl Stream<Item = T> + 'static) -> Self {
        Self {
            wrapper,
            stream: stream.boxed_local(),
        }
    }

    pub async fn step(&mut self) -> T {
        if let Some(t) = self.stream.next().await {
            t
        } else {
            std::future::pending().await
        }
    }
}
