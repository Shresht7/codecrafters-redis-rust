// OPCODES

pub enum OPCode {
    Aux = 0xfa,
    ResizeDB = 0xfb,
    ExpireTimeMs = 0xfc,
    ExpireTime = 0xfd,
    SelectDB = 0xfe,
    End = 0xff,
}
