mod ring;
mod ring_grpc;

use crate::ring::Empty;
use crate::ring_grpc::{Ring, RingServer};
use failure::Error;
use grpc::{Error as GrpcError, RequestOptions, ServerBuilder, SingleResponse};
use grpc_ring::Remote;
use log::{debug, trace};
use std::sync::{
  mpsc::{channel, Receiver, Sender},
  Mutex,
};
use std::{env, net::SocketAddr};

macro_rules! try_or_response {
  ($x:expr) => {{
    match $x {
      Ok(value) => value,
      Err(err) => {
        let error = GrpcError::Panic(err.to_string());
        SingleResponse::err(error)
      }
    }
  }};
}

struct RingImpl {
  // Since Ring trait requires the Sync trait for sending instance,
  // the best way is to wrap Sender with Mutex
  sender: Mutex<Sender<Action>>,
}

impl RingImpl {
  fn new(sender: Sender<Action>) -> Self {
    Self {
      sender: Mutex::new(sender),
    }
  }
  fn send_action(&self, action: Action) -> SingleResponse<Empty> {
    // try to lock mutex sender, and if fail, then return SingleResponse type error
    let tx = try_or_response!(self.sender.lock());
    try_or_response!(tx.send(action));
    let result = Empty::new();
    SingleResponse::completed(result)
  }
}

impl Ring for RingImpl {
  fn start_roll_call(&self, _: RequestOptions, _: Empty) -> SingleResponse<Empty> {
    trace!("START_ROLL_CALL");
    self.send_action(Action::StartRollCall)
  }
  fn mark_itself(&self, _: RequestOptions, _: Empty) -> SingleResponse<Empty> {
    trace!("MARK_ITSELF");
    self.send_action(Action::MarkItself)
  }
}

fn worker_loop(receiver: Receiver<Action>) -> Result<(), Error> {
  let next = env::var("NEXT")?.parse()?;
  let remote = Remote::new(next)?;
  let mut in_roll_call = false;
  for action in recevier.iter() {
    match action {}
  }
  Ok(())
}

fn main() -> Result<(), Error> {
    // Logger initialize to see results
    env_logger::init();
    let (tx, rx) = channel();
    // get Socket Address value from env var
    let addr: SocketAddr = env::var("ADDRESS")?.parse()?;
    // Since this is a practice, we create "new plain" server
    // which doesn't require TLS(SSL) certificate
    let mut server = ServerBuilder::new_plain();
    server.http.set_addr(addr)?;
    let ring = RingImpl::new(tx);
    server.add_service(RingServer::new_service_def(ring));
    server.http.set_cpu_pool_threads(4);
    let _server = server.build();

    worker_loop(rx)
}
