use std::io;
use std::time::Duration;
use icmpose::client::Client;

fn main() -> io::Result<()> {
  let mut client = Client::new()?;
  let dest_ip = "127.0.0.1";

  match client.ping(dest_ip, Duration::from_secs(1)) {
    Ok(resp_packet) => {
      resp_packet.print();
      Ok(())
    }
    Err(e) => {
      eprintln!("error: {}", e);
      Err(e)
    }
  }
}