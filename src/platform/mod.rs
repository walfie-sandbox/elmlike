mod mailbox;

pub use self::mailbox::{Inbox, Outbox};
use futures::{Async, Future, Stream};

pub trait Program {
    type Flags;
    type Model;
    type Msg;
    type Cmd;

    fn init(
        &self,
        flags: Self::Flags,
        messages: &Outbox<Self::Msg>,
        commands: &Outbox<Self::Cmd>,
    ) -> Self::Model;

    fn update(&mut self, model: &mut Self::Model, msg: Self::Msg, cmd_outbox: &Outbox<Self::Cmd>);
}

pub trait EffectManager {
    type Msg;
    type Cmd;

    fn handle(&mut self, cmd: Self::Cmd, msg_outbox: &Outbox<Self::Msg>);
}

#[derive(Debug)]
#[must_use = "futures do nothing unless polled"]
pub struct Worker<P, E>
where
    P: Program,
{
    program: P,
    effect_manager: E,
    model: P::Model,
    msg_inbox: Inbox<P::Msg>,
    cmd_inbox: Inbox<P::Cmd>,
    msg_outbox: Outbox<P::Msg>,
    cmd_outbox: Outbox<P::Cmd>,
}

impl<P, E> Worker<P, E>
where
    P: Program,
    E: EffectManager<Msg = P::Msg, Cmd = P::Cmd>,
{
    pub fn new(program: P, effect_manager: E, flags: P::Flags) -> Self {
        let (msg_outbox, msg_inbox) = mailbox::new();
        let (cmd_outbox, cmd_inbox) = mailbox::new();
        let model = program.init(flags, &msg_outbox, &cmd_outbox);

        Worker {
            program,
            effect_manager,
            model,
            msg_outbox,
            cmd_outbox,
            msg_inbox,
            cmd_inbox,
        }
    }
}

impl<P, E> Future for Worker<P, E>
where
    P: Program,
    E: EffectManager<Msg = P::Msg, Cmd = P::Cmd>,
{
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        // Process all `Cmd`s
        loop {
            match self.cmd_inbox.poll()? {
                Async::Ready(Some(cmd)) => {
                    self.effect_manager.handle(cmd, &self.msg_outbox);
                }
                _ => break,
            }
        }

        // Process all `Msg`s
        loop {
            match self.msg_inbox.poll()? {
                Async::Ready(Some(msg)) => {
                    self.program.update(&mut self.model, msg, &self.cmd_outbox);
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
