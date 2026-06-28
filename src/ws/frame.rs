//! WebSocket frame parser and writer

#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {
    Text,
    Binary,
    Close,
    Ping,
    Pong,
    Continue,
}

#[derive(Debug, Clone)]
pub struct WsFrame {
    pub opcode: OpCode,
    pub payload: Vec<u8>,
    pub fin: bool,
}

impl WsFrame {
    pub fn text(data: impl Into<String>) -> Self {
        Self {
            opcode: OpCode::Text,
            payload: data.into().into_bytes(),
            fin: true,
        }
    }

    pub fn binary(data: Vec<u8>) -> Self {
        Self {
            opcode: OpCode::Binary,
            payload: data,
            fin: true,
        }
    }

    pub fn close() -> Self {
        Self {
            opcode: OpCode::Close,
            payload: vec![],
            fin: true,
        }
    }

    pub fn pong(payload: Vec<u8>) -> Self {
        Self {
            opcode: OpCode::Pong,
            payload,
            fin: true,
        }
    }

    /// Raw bytes se WsFrame parse karo
    pub fn parse(buf: &[u8]) -> Option<(Self, usize)> {
        if buf.len() < 2 { return None; }

        let fin    = (buf[0] & 0x80) != 0;
        let opcode = match buf[0] & 0x0F {
            0x0 => OpCode::Continue,
            0x1 => OpCode::Text,
            0x2 => OpCode::Binary,
            0x8 => OpCode::Close,
            0x9 => OpCode::Ping,
            0xA => OpCode::Pong,
            _   => return None,
        };

        let masked         = (buf[1] & 0x80) != 0;
        let payload_len    = (buf[1] & 0x7F) as usize;

        let (payload_len, header_len) = match payload_len {
            126 => {
                if buf.len() < 4 { return None; }
                let len = u16::from_be_bytes([buf[2], buf[3]]) as usize;
                (len, 4)
            }
            127 => {
                if buf.len() < 10 { return None; }
                let len = u64::from_be_bytes([
                    buf[2], buf[3], buf[4], buf[5],
                    buf[6], buf[7], buf[8], buf[9],
                ]) as usize;
                (len, 10)
            }
            n => (n, 2),
        };

        let mask_len   = if masked { 4 } else { 0 };
        let total      = header_len + mask_len + payload_len;

        if buf.len() < total { return None; }

        let mut payload = buf[header_len + mask_len..total].to_vec();

        if masked {
            let mask = &buf[header_len..header_len + 4];
            for (i, byte) in payload.iter_mut().enumerate() {
                *byte ^= mask[i % 4];
            }
        }

        Some((Self { opcode, payload, fin }, total))
    }

    /// WsFrame ko raw bytes mein convert karo
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        let opcode_byte = match self.opcode {
            OpCode::Continue => 0x0,
            OpCode::Text     => 0x1,
            OpCode::Binary   => 0x2,
            OpCode::Close    => 0x8,
            OpCode::Ping     => 0x9,
            OpCode::Pong     => 0xA,
        };

        buf.push(if self.fin { 0x80 | opcode_byte } else { opcode_byte });

        let len = self.payload.len();
        if len < 126 {
            buf.push(len as u8);
        } else if len < 65536 {
            buf.push(126);
            buf.extend_from_slice(&(len as u16).to_be_bytes());
        } else {
            buf.push(127);
            buf.extend_from_slice(&(len as u64).to_be_bytes());
        }

        buf.extend_from_slice(&self.payload);
        buf
    }

    pub fn text_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.payload).ok()
    }
}