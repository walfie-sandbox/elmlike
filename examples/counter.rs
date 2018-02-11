// TODO
#![allow(dead_code)]

extern crate elmlike;

use elmlike::platform::*;

type Model = i32;

type Flags = ();

type Cmd = ();

enum Msg {
    Increase,
    Decrease,
}

struct Application;

impl Program for Application {
    type Flags = Flags;
    type Model = Model;
    type Msg = Msg;
    type Cmd = Cmd;

    fn init(&self, _flags: Self::Flags) -> (Self::Model, Option<Self::Cmd>) {
        (0, None)
    }

    fn update(&mut self, model: &mut Self::Model, msg: Self::Msg) -> Option<Self::Cmd> {
        match msg {
            Msg::Increase => {
                *model += 1;
            }
            Msg::Decrease => {
                *model -= 1;
            }
        }

        None
    }
}

fn main() {}
