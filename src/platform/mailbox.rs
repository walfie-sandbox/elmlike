use futures::{Async, Future, IntoFuture, Stream};
use futures::future::Executor;
use futures::sync::mpsc;
use tokio::executor::current_thread::task_executor;

pub(crate) fn new<T>() -> (Outbox<T>, Inbox<T>) {
    let (tx, rx) = mpsc::unbounded();
    (Outbox(tx), Inbox(rx))
}

#[derive(Debug)]
pub struct Outbox<T>(mpsc::UnboundedSender<T>);

impl<T> Clone for Outbox<T> {
    fn clone(&self) -> Self {
        Outbox(self.0.clone())
    }
}

impl<T> Outbox<T> {
    pub fn send(&self, msg: T) {
        let _ = self.0.unbounded_send(msg);
    }

    pub fn send_future<F>(&self, f: F)
    where
        F: IntoFuture<Item = T, Error = ()> + 'static,
        T: 'static,
    {
        let tx = self.clone();
        let send_future = f.into_future().map(move |item: T| tx.send(item));
        let _ = task_executor().execute(send_future); // TODO: log?
    }

    pub fn send_stream<S>(&self, s: S)
    where
        S: Stream<Item = T, Error = ()> + 'static,
        T: 'static,
    {
        let tx = self.clone();
        let send_stream = s.for_each(move |item: T| Ok(tx.send(item)));
        task_executor().execute(send_stream).unwrap(); // TODO: log?
    }
}

#[derive(Debug)]
pub struct Inbox<T>(mpsc::UnboundedReceiver<T>);

impl<T> Stream for Inbox<T> {
    type Item = T;
    type Error = ();

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        self.0.poll()
    }
}
