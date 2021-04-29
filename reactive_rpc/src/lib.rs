mod ring;
mod ring_grpc;

use crate::ring::Empty;
use crate::ring_grpc::{Ring, RingClient};
use grpc::{ClientConf, ClientStubExt, Error as GrpcError, RequestOptions};
use std::net::SocketAddr;

pub struct Remote {
  client: RingClient,
}

impl Remote {
  pub fn new(addr: SocketAddr) -> Result<Self, GrpcError> {
    // Socket Addr type "addr" includes ip address and port
    let host = addr.ip().to_string();
    let port = addr.port();
    let conf = ClientConf::default();
    let client = RingClient::new_plain(&host, port, conf)?;
    Ok(Self { client })
  }
  // Method to call remote methods
  pub fn start_roll_call(&self) -> Result<Empty, GrpcError> {
    self
      .client
      // Ring client contains "start roll call" method.
      .start_roll_call(RequestOptions::new(), Empty::new())
      .wait()
      .map(|(_, value, _)| value) // we hide other two data, because we are not interested
  }
  // Method to mark itself as task is done
  pub fn mark_itself(&self) -> Result<Empty, GrpcError> {
    self
      .client
      // Ring client contains "mark itself" method.
      .mark_itself(RequestOptions::new(), Empty::new())
      .wait()
      .map(|(_, value, _)| value)
  }
}
