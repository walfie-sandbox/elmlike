extern crate elmlike;
extern crate futures;
extern crate tokio;
extern crate tokio_core;

use elmlike::platform::*;
use elmlike::platform::router::*;
use futures::Stream;
use std::io::BufRead;
use std::time::Duration;
use tokio_core::reactor::{Core, Handle, Interval};

struct Model {
    count: i32,
}

struct Flags {
    initial: i32,
}

enum Cmd {
    Print(i32),
}

enum Msg {
    Increase,
    Decrease,
    Print,
}

struct Application {
    handle: Handle,
}

impl Program for Application {
    type Flags = Flags;
    type Model = Model;
    type Msg = Msg;
    type Cmd = Cmd;

    fn init(
        &self,
        flags: Self::Flags,
        messages: &AsyncRouter<Self::Msg>,
        commands: &AsyncRouter<Self::Cmd>,
    ) -> Self::Model {
        commands.send(Cmd::Print(flags.initial));

        // Subscription that ticks once per second
        let ticks = Interval::new(Duration::from_secs(1), &self.handle)
            .unwrap()
            .map_err(|_| ())
            .map(|_| Msg::Print);

        messages.send_stream(ticks);

        let router = messages.clone();
        std::thread::spawn(move || loop {
            let input = std::io::stdin();

            for line in input.lock().lines() {
                for c in line.expect("failed to get line").chars() {
                    match c {
                        '+' => router.send(Msg::Increase),
                        '-' => router.send(Msg::Decrease),
                        _ => {}
                    }
                }

                router.send(Msg::Print);
            }
        });

        Model {
            count: flags.initial,
        }
    }

    fn update(
        &mut self,
        model: &mut Self::Model,
        msg: Self::Msg,
        commands: &AsyncRouter<Self::Cmd>,
    ) {
        match msg {
            Msg::Increase => {
                model.count += 1;
            }
            Msg::Decrease => {
                model.count -= 1;
            }
            Msg::Print => commands.send(Cmd::Print(model.count)),
        }
    }
}

struct Effects;

impl EffectManager for Effects {
    type Msg = Msg;
    type Cmd = Cmd;

    fn handle(&mut self, cmd: Self::Cmd, _router: &AsyncRouter<Self::Msg>) {
        match cmd {
            Cmd::Print(value) => {
                println!("{}", value);
            }
        }
    }
}

fn main() {
    let core = Core::new().unwrap();
    let app = Application {
        handle: core.handle(),
    };

    let flags = Flags { initial: 0 };

    use tokio::executor::current_thread;
    current_thread::run(move |_| {
        let worker = AsyncWorker::new(app, Effects, flags);
        current_thread::spawn(worker)
    });
}
