mod router;

use self::router::{AsyncReceiver, AsyncRouter, Router};
use futures::{Async, Future, Stream};
use futures::sync::mpsc;

pub trait Program {
    type Flags;
    type Model;
    type Msg;
    type Cmd;

    fn init(&self, flags: Self::Flags) -> (Self::Model, Option<Self::Cmd>);

    fn update(&mut self, model: &mut Self::Model, msg: Self::Msg) -> Option<Self::Cmd>;
}

pub trait EffectManager {
    type Msg;
    type Cmd;

    fn handle(&mut self, cmd: Self::Cmd, msg_router: AsyncRouter<Self::Msg>);
}

pub struct AsyncWorker<Msg, Cmd, P, E>
where
    P: Program,
    E: EffectManager,
{
    program: P,
    effect_manager: E,
    model: P::Model,
    msg_receiver: AsyncReceiver<Msg>,
    cmd_receiver: AsyncReceiver<Cmd>,
    msg_router: AsyncRouter<Msg>,
    cmd_router: AsyncRouter<Cmd>,
}

impl<Msg, Cmd, P, E> AsyncWorker<Msg, Cmd, P, E>
where
    P: Program<Msg = Msg, Cmd = Cmd>,
    E: EffectManager<Msg = Msg, Cmd = Cmd>,
{
    fn new(program: P, effect_manager: E, flags: P::Flags) -> Self {
        let (model, cmd) = program.init(flags);
        let (msg_router, msg_receiver) = router::async();
        let (cmd_router, cmd_receiver) = router::async();

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
}

impl<Msg, Cmd, P, E> Future for AsyncWorker<Msg, Cmd, P, E>
where
    P: Program<Msg = Msg, Cmd = Cmd>,
    E: EffectManager<Msg = Msg, Cmd = Cmd>,
{
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        unimplemented!()
    }
}
