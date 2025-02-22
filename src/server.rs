use std::{io, mem};
use std::net::Ipv4Addr;
use libc::{self, close, sa_family_t, sockaddr, sockaddr_in, socket,
           AF_INET, IPPROTO_ICMP, SOCK_RAW, c_void, recvfrom};

use crate::icmp_packet::IcmpPacket;

pub struct Server {
  socket: i32,
}

impl Server {
  pub fn new() -> io::Result<Self> {
    let socket = unsafe { socket(AF_INET, SOCK_RAW, IPPROTO_ICMP) };
    if socket < 0 {
      return Err(io::Error::last_os_error());
    }

    let addr = sockaddr_in {
      sin_family: AF_INET as sa_family_t,
      sin_port: 0,
      sin_addr: libc::in_addr { s_addr: 0 },
      sin_zero: [0; 8],
      sin_len: 0,
    };

    let bind_result = unsafe {
      libc::bind(
        socket,
        &addr as *const _ as *const sockaddr,
        mem::size_of::<sockaddr_in>() as u32,
      )
    };

    if bind_result < 0 {
      return Err(io::Error::last_os_error());
    }

    Ok(Self { socket })
  }

  pub fn listen(&self) -> io::Result<()> {
    println!("Listening for ICMP packets...");
    let mut buffer = vec![0u8; 1024];

    loop {
      let mut src_addr: sockaddr_in = unsafe { mem::zeroed() };
      let mut src_addr_len = mem::size_of::<sockaddr_in>() as u32;

      let received = unsafe {
        recvfrom(
          self.socket,
          buffer.as_mut_ptr() as *mut c_void,
          buffer.len(),
          0,
          &mut src_addr as *mut _ as *mut sockaddr,
          &mut src_addr_len,
        )
      };

      if received < 0 {
        return Err(io::Error::last_os_error());
      }

      let src_ip = Ipv4Addr::from(u32::from_be(src_addr.sin_addr.s_addr));
      let ip_header_length = (buffer[0] & 0x0F) * 4;
      let icmp_data = &buffer[ip_header_length as usize..received as usize];

      if let Some(packet) = IcmpPacket::from_bytes(icmp_data) {
        println!("Received from {}:", src_ip);
        packet.print();
      }
    }
  }
}

impl Drop for Server {
  fn drop(&mut self) {
    unsafe { close(self.socket) };
  }
}