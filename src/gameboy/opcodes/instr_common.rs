use crate::gameboy::Gameboy;
use core::num::Wrapping as W;

#[inline(always)]
pub fn resolve_read_reg_low(gb: &mut Gameboy, opcode: W<u8>) -> W<u8> {
    match opcode.0 & 0x7 {
        0x0 => {
            return gb.reg.b;
        }
        0x1 => {
            return gb.reg.c;
        }
        0x2 => {
            return gb.reg.d;
        }
        0x3 => {
            return gb.reg.e;
        }
        0x4 => {
            return gb.reg.h;
        }
        0x5 => {
            return gb.reg.l;
        }
        0x6 => {
            return gb.read_byte(gb.reg.get_hl());
        }
        0x7 => {
            return gb.reg.a;
        }
        _ => {
            panic!("impossible???");
        }
    }
}

#[inline(always)]
pub fn resolve_read_reg_high(gb: &mut Gameboy, opcode: W<u8>) -> W<u8> {
    match (opcode >> 3).0 & 0x7 {
        0x0 => {
            return gb.reg.b;
        }
        0x1 => {
            return gb.reg.c;
        }
        0x2 => {
            return gb.reg.d;
        }
        0x3 => {
            return gb.reg.e;
        }
        0x4 => {
            return gb.reg.h;
        }
        0x5 => {
            return gb.reg.l;
        }
        0x6 => {
            return gb.read_byte(gb.reg.get_hl());
        }
        0x7 => {
            return gb.reg.a;
        }
        _ => {
            panic!("impossible???");
        }
    }
}

#[inline(always)]
pub fn resolve_write_reg_low(gb: &mut Gameboy, opcode: W<u8>, value: W<u8>) {
    match opcode.0 & 0x7 {
        0x0 => {
            gb.reg.b = value;
        }
        0x1 => {
            gb.reg.c = value;
        }
        0x2 => {
            gb.reg.d = value;
        }
        0x3 => {
            gb.reg.e = value;
        }
        0x4 => {
            gb.reg.h = value;
        }
        0x5 => {
            gb.reg.l = value;
        }
        0x6 => {
            return gb.write_byte(gb.reg.get_hl(), value);
        }
        0x7 => {
            gb.reg.a = value;
        }
        _ => {
            panic!("impossible???");
        }
    }
}

#[inline(always)]
pub fn resolve_write_reg_high(gb: &mut Gameboy, opcode: W<u8>, value: W<u8>) {
    match (opcode >> 3).0 & 0x7 {
        0x0 => {
            gb.reg.b = value;
        }
        0x1 => {
            gb.reg.c = value;
        }
        0x2 => {
            gb.reg.d = value;
        }
        0x3 => {
            gb.reg.e = value;
        }
        0x4 => {
            gb.reg.h = value;
        }
        0x5 => {
            gb.reg.l = value;
        }
        0x6 => {
            return gb.write_byte(gb.reg.get_hl(), value);
        }
        0x7 => {
            gb.reg.a = value;
        }
        _ => {
            panic!("impossible???");
        }
    }
}
