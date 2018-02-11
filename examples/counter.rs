// TODO
#![allow(dead_code)]

extern crate elmlike;
extern crate tokio_core;

use elmlike::platform::*;
use elmlike::platform::router::*;
use tokio_core::reactor::Core;

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

struct Application;

impl Program for Application {
    type Flags = Flags;
    type Model = Model;
    type Msg = Msg;
    type Cmd = Cmd;

    fn init(&self, flags: Self::Flags, commands: &AsyncRouter<Self::Cmd>) -> Self::Model {
        commands.send(Cmd::Print(flags.initial));

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

    fn handle(&mut self, cmd: Self::Cmd, _msg_router: &AsyncRouter<Self::Msg>) {
        match cmd {
            Cmd::Print(value) => {
                println!("{}", value);
            }
        }
    }
}

fn main() {
    let flags = Flags { initial: 0 };
    let worker = AsyncWorker::new(Application, Effects, flags);

    let msg_router = worker.msg_router();

    use std::io::BufRead;

    std::thread::spawn(move || loop {
        let input = std::io::stdin();

        for line in input.lock().lines() {
            for c in line.expect("failed to get line").chars() {
                match c {
                    '+' => msg_router.send(Msg::Increase),
                    '-' => msg_router.send(Msg::Decrease),
                    _ => {}
                }
            }

            msg_router.send(Msg::Print);
        }
    });

    let mut core = Core::new().unwrap();
    core.run(worker).unwrap();
}
