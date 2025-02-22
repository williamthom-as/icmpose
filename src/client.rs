use std::ffi::c_void;
use std::io;
use std::net::Ipv4Addr;
use std::time::Duration;
use libc::{close, recvfrom, sa_family_t, sendto, sockaddr, sockaddr_in, socket, AF_INET, IPPROTO_ICMP, SOCK_RAW};
use std::mem::size_of;

use crate::icmp_packet::IcmpPacket;

pub struct Client {
  socket: i32,
  sequence: u16,
  identifier: u16,
}

impl Client {
  pub fn new() -> io::Result<Self> {
    let socket = unsafe { socket(AF_INET, SOCK_RAW, IPPROTO_ICMP) };
    if socket < 0 {
      return Err(io::Error::last_os_error());
    }

    Ok(Self {
      socket,
      sequence: 0,
      identifier: std::process::id() as u16,
    })
  }

  pub fn ping(&mut self, dest_ip: &str, timeout: Duration) -> io::Result<IcmpPacket> {
    let addr = dest_ip.parse::<Ipv4Addr>()
      .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    let mut packet = IcmpPacket::new_echo_req(self.identifier, self.sequence);
    self.sequence = self.sequence.wrapping_add(1);

    let dest_addr = sockaddr_in {
      sin_family: AF_INET as sa_family_t,
      sin_port: 0,
      sin_addr: libc::in_addr {
        s_addr: u32::from_be_bytes(addr.octets())
      },
      sin_zero: [0; 8],
      sin_len: 0,
    };

    // convert to bytes and send packet then check resp
    let bytes = packet.bytes();
    let sent = unsafe {
      sendto(
        self.socket,
        bytes.as_ptr() as *const c_void,
        bytes.len(),
        0,
        &dest_addr as *const _ as *const sockaddr,
        size_of::<sockaddr_in>() as u32,
      )
    };

    if sent < 0 {
      return Err(io::Error::last_os_error());
    }

    // handle socket timeout
    unsafe {
      let tv = libc::timeval {
        tv_sec: timeout.as_secs() as i64,
        tv_usec: timeout.subsec_micros() as i32,
      };

      libc::setsockopt(
        self.socket,
        libc::SOL_SOCKET,
        libc::SO_RCVTIMEO,
        &tv as *const _ as *const c_void,
        size_of::<libc::timeval>() as u32,
      );
    }

    // get resp
    let mut buf = vec![0u8; 1024];
    let received = unsafe {
      recvfrom(
        self.socket,
        buf.as_mut_ptr() as *mut c_void,
        buf.len(),
        0,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
      )
    };

    if received < 0 {
      return Err(io::Error::last_os_error());
    }

    let ip_header_length = (buf[0] & 0x0F) * 4;
    let icmp_packet = &buf[ip_header_length as usize..received as usize];

    IcmpPacket::from_bytes(icmp_packet)
      .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid ICMP packet returned"))
  }
}

impl Drop for Client {
  fn drop(&mut self) {
    unsafe { close(self.socket) };
  }
}