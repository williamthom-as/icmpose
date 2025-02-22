use std::io;
use icmpose::server::Server;

fn main() -> io::Result<()> {
  let server = Server::new()?;
  server.listen()
}