extern crate elmlike;
extern crate futures;
extern crate hyper;
extern crate tokio;

use elmlike::platform::*;
use futures::{Future, Stream};
use hyper::header::ContentLength;
use hyper::server::{Http, Request, Response, Service};

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

struct Server;

impl EffectManager for Server {
    type Msg = Msg;
    type Cmd = Cmd;

    fn handle(&mut self, cmd: Self::Cmd, _msg_outbox: &Outbox<Self::Msg>) {}
}

const PHRASE: &'static str = "HELLO WORLD";
impl Service for Server {
    // boilerplate hooking up hyper's server types
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    // The future representing the eventual Response your call will
    // resolve to. This can change to whatever Future you need.
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, _req: Request) -> Self::Future {
        // We're currently ignoring the Request
        // And returning an 'ok' Future, which means it's ready
        // immediately, and build a Response with the 'PHRASE' body.
        Box::new(futures::future::ok(
            Response::new()
                .with_header(ContentLength(PHRASE.len() as u64))
                .with_body(PHRASE),
        ))
    }
}

fn main() {
    let addr = "127.0.0.1:3000".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(&addr).expect("failed to bind TcpListener");

    let server = std::rc::Rc::new(Server);

    let flags = ();

    use tokio::executor::current_thread;
    current_thread::run(move |_| {
        let http = Http::<hyper::Chunk>::new();
        let server_future = listener
            .incoming()
            .for_each(move |sock| {
                let conn = http.serve_connection(sock, server.clone())
                    .map(|_| ())
                    .map_err(|_| ());
                current_thread::spawn(conn);
                Ok(())
            })
            .map_err(|_| ());
        current_thread::spawn(server_future);

        let worker = Worker::new(Application, Server, flags);
        current_thread::spawn(worker);

        println!("Listening on {}", addr);
    });
}
