mod router;

use futures::sync::mpsc;
use self::router::{AsyncRouter, Router};
use futures::{Future, Stream};

pub trait Program {
    type Flags;
    type Model;
    type Msg;
    type Cmd;

    fn init(&self, flags: Self::Flags) -> (Self::Model, Option<Self::Cmd>);

    fn update(&mut self, model: &mut Self::Model, msg: Self::Msg) -> Option<Self::Cmd>;
}

pub struct AsyncWorker<P: Program> {
    program: P,
    model: P::Model,
    msg_router: AsyncRouter<P::Msg>,
    cmd_router: AsyncRouter<P::Cmd>,
}

impl<P: Program> AsyncWorker<P> {
    fn new(program: P, flags: P::Flags) -> Self {
        let (model, cmd) = program.init(flags);
        let (msg_router, msg_receiver) = router::async();
        let (cmd_router, cmd_receiver) = router::async();

        AsyncWorker {
            program,
            model,
            msg_router,
            cmd_router,
        }
    }
}
