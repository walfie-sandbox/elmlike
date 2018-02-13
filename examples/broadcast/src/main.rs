// This example is incomplete and I'm too lazy to fix it for now
extern crate elmlike;
extern crate futures;
extern crate hyper;
extern crate tokio;

use elmlike::platform::*;
use futures::{Future, Stream};
use futures::sync::mpsc;
use hyper::Chunk;
use hyper::header::ContentLength;
use hyper::server::{Http, Request, Response, Service};
use std::cell::RefCell;

struct Model {
    messages: Vec<String>,
}

type Flags = ();

enum Cmd {
    Broadcast(String),
}

enum Msg {
    UserConnected { name: String },
    UserDisconnected { name: String },
    Input(String),
}

struct Application;

impl Program for Application {
    type Flags = Flags;
    type Model = Model;
    type Msg = Msg;
    type Cmd = Cmd;

    fn init(
        &self,
        flags: Self::Flags,
        messages: &Outbox<Self::Msg>,
        commands: &Outbox<Self::Cmd>,
    ) -> Self::Model {
        Model { messages: vec![] }
    }

    fn update(&mut self, model: &mut Self::Model, msg: Self::Msg, cmd_outbox: &Outbox<Self::Cmd>) {
        use Msg::*;

        match msg {
            UserConnected { name } => cmd_outbox.send(Cmd::Broadcast(format!("{} joined.", name))),
            UserDisconnected { name } => {
                cmd_outbox.send(Cmd::Broadcast(format!("{} joined.", name)))
            }
            Input(text) => cmd_outbox.send(Cmd::Broadcast(text)),
        }
    }
}

struct Effects {}

impl EffectManager for Effects {
    type Msg = Msg;
    type Cmd = Cmd;

    fn handle(&mut self, cmd: Self::Cmd, _msg_outbox: &Outbox<Self::Msg>) {}
}

struct Broadcast {
    name: String,
    rx: RefCell<Option<mpsc::Receiver<Result<Chunk, hyper::Error>>>>,
}

const PHRASE: &'static str = "HELLO WORLD";
impl Service for Broadcast {
    // boilerplate hooking up hyper's server types
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    // The future representing the eventual Response your call will
    // resolve to. This can change to whatever Future you need.
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, _req: Request) -> Self::Future {
        Box::new(futures::future::ok(
            Response::new()
                .with_header(ContentLength(PHRASE.len() as u64))
                .with_body(self.rx.borrow_mut().take().unwrap()),
        ))
    }
}

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(&addr).expect("failed to bind TcpListener");

    let effects = Effects {};

    let flags = ();

    use tokio::executor::current_thread;
    current_thread::run(move |_| {
        let http = Http::<Chunk>::new();
        let server_future = listener
            .incoming()
            .for_each(move |sock| {
                let remote_addr = sock.peer_addr().expect("peer_addr");
                let (tx, rx) = mpsc::channel(1024);
                let broadcast = Broadcast {
                    name: remote_addr.to_string(),
                    rx: RefCell::new(Some(rx)),
                };
                let conn = http.serve_connection(sock, broadcast)
                    .map(|_| ())
                    .map_err(|_| ());
                current_thread::spawn(conn);
                Ok(())
            })
            .map_err(|_| ());
        current_thread::spawn(server_future);

        let worker = Worker::new(Application, effects, flags);
        current_thread::spawn(worker);

        println!("Listening on {}", addr);
    });
}
