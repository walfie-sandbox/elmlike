use futures::{Async, Poll, Sink, StartSend, Stream};
use futures::sync::mpsc;

pub trait Router<T> {
    fn send(&self, msg: T);
}

pub fn async<T>() -> (AsyncRouter<T>, AsyncReceiver<T>) {
    let (tx, rx) = mpsc::unbounded();
    (AsyncRouter(tx), AsyncReceiver(rx))
}

#[derive(Debug)]
pub struct AsyncRouter<T>(mpsc::UnboundedSender<T>);

impl<T> Clone for AsyncRouter<T> {
    fn clone(&self) -> Self {
        AsyncRouter(self.0.clone())
    }
}

impl<T> Router<T> for AsyncRouter<T> {
    fn send(&self, msg: T) {
        let _ = self.0.unbounded_send(msg);
    }
}

impl<T> Sink for AsyncRouter<T> {
    type SinkItem = T;
    type SinkError = ();

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        self.0.start_send(item).map_err(|_| ())
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        self.0.poll_complete().map_err(|_| ())
    }
}

#[derive(Debug)]
pub struct AsyncReceiver<T>(mpsc::UnboundedReceiver<T>);

impl<T> Stream for AsyncReceiver<T> {
    type Item = T;
    type Error = ();

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        self.0.poll()
    }
}
