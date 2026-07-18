use crate::cpu::AddressingMode;

struct OpInfo {
    mnemonic: &'static str,
    mode: AddressingMode,
    length: u8
}

const OPCODE_TABLE: [Option<OpInfo>; 256] = {
    let mut table: [Option<OpInfo>; 256] = [None; 256];
    table
};

fn decode_opcode(opcode: u8) -> Option<(&'static str, AddressingMode, u8)> {
    use AddressingMode::*;
    match opcode {
        
    }
}