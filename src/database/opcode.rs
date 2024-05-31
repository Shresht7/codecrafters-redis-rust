// OPCODES

pub enum OPCode {
    Aux = 0xfa,
    ResizeDB = 0xfb,
    ExpireTimeMs = 0xfc,
    ExpireTime = 0xfd,
    SelectDB = 0xfe,
    End = 0xff,
}

impl From<u8> for OPCode {
    fn from(byte: u8) -> Self {
        match byte {
            0xfa => OPCode::Aux,
            0xfb => OPCode::ResizeDB,
            0xfc => OPCode::ExpireTimeMs,
            0xfd => OPCode::ExpireTime,
            0xfe => OPCode::SelectDB,
            0xff => OPCode::End,
            _ => {
                panic!("Invalid Opcode {}", byte)
            }
        }
    }
}

impl OPCode {
    pub fn is_opcode(byte: u8) -> bool {
        byte == OPCode::Aux as u8
            || byte == OPCode::ResizeDB as u8
            || byte == OPCode::ExpireTimeMs as u8
            || byte == OPCode::ExpireTime as u8
            || byte == OPCode::SelectDB as u8
            || byte == OPCode::End as u8
    }
}
