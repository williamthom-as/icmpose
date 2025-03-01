#[derive(Debug, Clone)]
pub struct IcmpPacket {
  type_: u8,
  code: u8,
  checksum: u16,
  identifier: u16,
  sequence: u16,
  payload: Vec<u8>,
}

impl IcmpPacket {
  pub fn new_echo_req(identifier: u16, sequence: u16) -> Self {
    Self {
      type_: 8,
      code: 0,
      checksum: 0,
      identifier,
      sequence,
      payload: vec![0; 56], // move out to func arg
    }
  }

  pub fn make_echo_resp(&self) -> Self {
    Self {
      type_: 0,
      code: 0,
      checksum: 0,
      identifier: self.identifier,
      sequence: self.sequence,
      payload: self.payload.clone(),
    }
  }

  pub fn is_echo_req(&self) -> bool {
    self.type_ == 8 && self.code == 0
  }

  pub fn bytes(&mut self) -> Vec<u8> {
    self.checksum = self.calculate_checksum();

    let mut bytes = Vec::with_capacity(8 + self.payload.len());

    bytes.push(self.type_);
    bytes.push(self.code);
    bytes.extend_from_slice(&self.checksum.to_be_bytes());
    bytes.extend_from_slice(&self.identifier.to_be_bytes());
    bytes.extend_from_slice(&self.sequence.to_be_bytes());
    bytes.extend(&self.payload);
    bytes
  }

  pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
    if bytes.len() < 8 {
      return None;
    }

    Some(Self {
      type_: bytes[0],
      code: bytes[1],
      checksum: u16::from_be_bytes([bytes[2], bytes[3]]),
      identifier: u16::from_be_bytes([bytes[4], bytes[5]]),
      sequence: u16::from_be_bytes([bytes[6], bytes[7]]),
      payload: bytes[8..].to_vec(),
    })
  }

  pub fn print(&self) {
    println!("ICMP packet:");
    println!("  type: {}", self.type_);
    println!("  code: {}", self.code);
    println!("  checksum: 0x{:04x}", self.checksum);
    println!("  identifier: {}", self.identifier);
    println!("  sequence: {}", self.sequence);
    println!("  payload (bytes): {} bytes", self.payload.len());
    println!("  payload (content): {:?} bytes", self.payload);
  }

  fn calculate_checksum(&self) -> u16 {
    let mut sum = 0u32;

    // add headers
    sum += ((self.type_ as u16) << 8 | self.code as u16) as u32;
    sum += self.checksum as u32;
    sum += self.identifier as u32;
    sum += self.sequence as u32;

    // add payload/s
    let mut i = 0;
    while i < self.payload.len() {
      let word = if i + 1 < self.payload.len() {
        ((self.payload[i] as u16) << 8) | self.payload[i + 1] as u16
      } else {
        (self.payload[i] as u16) << 8
      };
      sum += word as u32;
      i += 2;
    }

    while (sum >> 16) != 0 {
      sum = (sum & 0xFFFF) + (sum >> 16);
    }

    // ones complement and trunc to u16 in case overflow
    !sum as u16
  }
}