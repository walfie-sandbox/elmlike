pub mod adapter;

use futures::{Async, Future, Stream};
use futures::sync::mpsc as sync_mpsc;

pub type MsgQueue<Msg> = UnboundedSink<Msg>;
pub type CmdSink<Cmd> = UnboundedSink<Cmd>;

pub struct UnboundedSink<T>(sync_mpsc::UnboundedSender<T>);

impl<T> UnboundedSink<T> {
    pub fn send(&self, msg: T) {
        let _ = self.0.unbounded_send(msg);
    }
}

impl<T> Clone for UnboundedSink<T> {
    fn clone(&self) -> Self {
        UnboundedSink(self.0.clone())
    }
}

pub trait CmdHandler<C> {
    fn handle(&mut self, cmd: C);
}

pub trait Component {
    type Msg;
    type Cmd;

    fn update(&mut self, msg: Self::Msg, cmd_sink: &CmdSink<Self::Cmd>);
}

struct App<C: Component, CH> {
    component: C,
    msg_sink: MsgQueue<C::Msg>,
    cmd_sink: CmdSink<C::Cmd>,
    msg_source: sync_mpsc::UnboundedReceiver<C::Msg>,
    cmd_source: sync_mpsc::UnboundedReceiver<C::Cmd>,
    cmd_handler: CH,
}

impl<C: Component, CH> App<C, CH> {
    fn new(component: C, cmd_handler: CH) -> Self {
        let (msg_tx, msg_rx) = sync_mpsc::unbounded();
        let (cmd_tx, cmd_rx) = sync_mpsc::unbounded();

        App {
            component,
            msg_source: msg_rx,
            cmd_source: cmd_rx,
            msg_sink: UnboundedSink(msg_tx),
            cmd_sink: UnboundedSink(cmd_tx),
            cmd_handler,
        }
    }

    fn queue(&self) -> MsgQueue<C::Msg> {
        self.msg_sink.clone()
    }
}

/*
impl<C, CH> Future for App<C, CH>
where
    C: Component,
{
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        loop {
            if let Some(event) = try_ready!(self.events.poll()) {}
        }
        unimplemented!();
    }
}
*/
