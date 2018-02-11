pub mod router;

use self::router::{AsyncReceiver, AsyncRouter};
use futures::{Async, Future, Stream};

pub trait Program {
    type Flags;
    type Model;
    type Msg;
    type Cmd;

    fn init(&self, flags: Self::Flags, commands: &AsyncRouter<Self::Cmd>) -> Self::Model;

    fn update(
        &mut self,
        model: &mut Self::Model,
        msg: Self::Msg,
        commands: &AsyncRouter<Self::Cmd>,
    );
}

pub trait EffectManager {
    type Msg;
    type Cmd;

    fn handle(&mut self, cmd: Self::Cmd, msg_router: &AsyncRouter<Self::Msg>);
}

#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct AsyncWorker<Msg, Cmd, P, E, S>
where
    P: Program,
{
    program: P,
    effect_manager: E,
    model: P::Model,
    msg_receiver: ::futures::stream::Select<AsyncReceiver<Msg>, S>,
    cmd_receiver: AsyncReceiver<Cmd>,
    msg_router: AsyncRouter<Msg>,
    cmd_router: AsyncRouter<Cmd>,
}

impl<Msg, Cmd, P, E, S> AsyncWorker<Msg, Cmd, P, E, S>
where
    P: Program<Msg = Msg, Cmd = Cmd>,
    E: EffectManager<Msg = Msg, Cmd = Cmd>,
    S: Stream<Item = Msg, Error = ()>,
{
    pub fn new(program: P, effect_manager: E, subscription: S, flags: P::Flags) -> Self {
        let (msg_router, msg_receiver) = router::async();
        let (cmd_router, cmd_receiver) = router::async();
        let model = program.init(flags, &cmd_router);

        AsyncWorker {
            program,
            effect_manager,
            model,
            msg_router,
            cmd_router,
            msg_receiver: msg_receiver.select(subscription),
            cmd_receiver,
        }
    }

    pub fn router(&self) -> AsyncRouter<Msg> {
        self.msg_router.clone()
    }
}

impl<Msg, Cmd, P, E, S> Future for AsyncWorker<Msg, Cmd, P, E, S>
where
    P: Program<Msg = Msg, Cmd = Cmd>,
    E: EffectManager<Msg = Msg, Cmd = Cmd>,
    S: Stream<Item = Msg, Error = ()>,
{
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        // Process all `Cmd`s
        loop {
            match self.cmd_receiver.poll()? {
                Async::Ready(Some(cmd)) => {
                    self.effect_manager.handle(cmd, &self.msg_router);
                }
                _ => break,
            }
        }

        // Process all `Msg`s
        loop {
            match self.msg_receiver.poll()? {
                Async::Ready(Some(msg)) => {
                    self.program.update(&mut self.model, msg, &self.cmd_router);
                }
                Async::Ready(None) => {
                    return Ok(Async::Ready(()));
                }
                Async::NotReady => {
                    return Ok(Async::NotReady);
                }
            }
        }
    }
}
