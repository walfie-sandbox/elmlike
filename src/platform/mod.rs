pub mod router;

use self::router::{AsyncReceiver, AsyncRouter};
use futures::{Async, Future, Stream};

pub trait Program {
    type Flags;
    type Model;
    type Msg;
    type Cmd;

    fn init(
        &self,
        flags: Self::Flags,
        messages: &AsyncRouter<Self::Msg>,
        commands: &AsyncRouter<Self::Cmd>,
    ) -> Self::Model;

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
pub struct AsyncWorker<P, E>
where
    P: Program,
{
    program: P,
    effect_manager: E,
    model: P::Model,
    msg_receiver: AsyncReceiver<P::Msg>,
    cmd_receiver: AsyncReceiver<P::Cmd>,
    msg_router: AsyncRouter<P::Msg>,
    cmd_router: AsyncRouter<P::Cmd>,
}

impl<P, E> AsyncWorker<P, E>
where
    P: Program,
    E: EffectManager<Msg = P::Msg, Cmd = P::Cmd>,
{
    pub fn new(program: P, effect_manager: E, flags: P::Flags) -> Self {
        let (msg_router, msg_receiver) = router::async();
        let (cmd_router, cmd_receiver) = router::async();
        let model = program.init(flags, &msg_router, &cmd_router);

        AsyncWorker {
            program,
            effect_manager,
            model,
            msg_router,
            cmd_router,
            msg_receiver,
            cmd_receiver,
        }
    }

    pub fn router(&self) -> AsyncRouter<P::Msg> {
        self.msg_router.clone()
    }
}

impl<P, E> Future for AsyncWorker<P, E>
where
    P: Program,
    E: EffectManager<Msg = P::Msg, Cmd = P::Cmd>,
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
