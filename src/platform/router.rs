use futures::{Async, Future, Stream};
use futures::future::Executor;
use futures::sync::mpsc;
use tokio::executor::current_thread::task_executor;

pub trait Router<T> {
    fn send(&self, msg: T);

    fn send_future<F>(&self, f: F)
    where
        F: Future<Item = T, Error = ()> + 'static,
        T: 'static;

    fn send_stream<S>(&self, s: S)
    where
        S: Stream<Item = T, Error = ()> + 'static,
        T: 'static;
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

    fn send_future<F>(&self, f: F)
    where
        F: Future<Item = T, Error = ()> + 'static,
        T: 'static,
    {
        let tx = self.clone();
        let send_future = f.map(move |item: T| tx.send(item));
        let _ = task_executor().execute(send_future); // TODO: log?
    }

    fn send_stream<S>(&self, s: S)
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
pub struct AsyncReceiver<T>(mpsc::UnboundedReceiver<T>);

impl<T> Stream for AsyncReceiver<T> {
    type Item = T;
    type Error = ();

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        self.0.poll()
    }
}
