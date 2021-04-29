use failure::Error; // To use universal type of Errors
use grpc_ring::Remote;
use std::env;

fn main() -> Result<(), Error> {
  // We parse env var called "NEXT", but we are not sure about the
  // result so we put question mark(?) end of the statement
  let next = env::var("NEXT")?.parse()?;
  // next provides an address to remote grpc
  let remote = Remote::new(next);
  // Perform a call!
  remote.start_roll_call()?;
  Ok(())
}
