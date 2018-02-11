#![allow(dead_code)]
extern crate elmlike;

use elmlike::framework::*;

struct Counter {
    count: i32,
}

impl Component for Counter {
    type Msg = Msg;
    type Cmd = Cmd;

    fn update(&mut self, msg: Self::Msg, cmd_sink: &CmdSink<Cmd>) {
        use Msg::*;

        match msg {
            Increase => {
                self.count += 1;
            }
            Decrease => {
                self.count -= 1;
            }
            Reset => {
                self.count = 0;
            }
            Print => {
                cmd_sink.send(Cmd::Print(self.count));
            }
        }
    }
}

enum Msg {
    Increase,
    Decrease,
    Reset,
    Print,
}

enum Cmd {
    Print(i32),
}

fn main() {}
