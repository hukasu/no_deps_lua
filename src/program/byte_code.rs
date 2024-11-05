#[derive(Debug, PartialEq)]
pub enum ByteCode {
    GetGlobal(u8, u8),
    LoadConstant(u8, u8),
    Call(u8, u8),
}
