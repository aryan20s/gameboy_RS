use crate::gameboy::Gameboy;
use core::num::Wrapping as W;

use super::instr_common;

use log::error;

#[inline(always)]
pub fn prefix_cb(gb: &mut Gameboy) -> bool {
    let opcode = gb.read_byte_inc_pc();

    match opcode.0 {
        0x00..=0x07 => {
            rlc_reg(gb, opcode);
        }
        0x08..=0x0f => {
            rrc_reg(gb, opcode);
        }
        0x10..=0x17 => {
            rl_reg(gb, opcode);
        }
        0x18..=0x1f => {
            rr_reg(gb, opcode);
        }
        0x20..=0x27 => {
            sla_reg(gb, opcode);
        }
        0x28..=0x2f => {
            sra_reg(gb, opcode);
        }
        0x30..=0x37 => {
            swap_reg(gb, opcode);
        }
        0x38..=0x3f => {
            srl_reg(gb, opcode);
        }
        0x40..=0x7f => {
            bit_n_reg(gb, opcode);
        }
        0x80..=0xbf => {
            res_n_reg(gb, opcode);
        }
        0xc0..=0xff => {
            set_n_reg(gb, opcode);
        }
    }

    return true;
}

#[inline(always)]
pub fn set_n_reg(gb: &mut Gameboy, opcode: W<u8>) {
    let bitmask = W(1u8.wrapping_shl((opcode >> 3).0 as u32 & 0x7));
    let lhs = instr_common::resolve_read_reg_low(gb, opcode);
    let value = lhs | bitmask;
    instr_common::resolve_write_reg_low(gb, opcode, value);
}

#[inline(always)]
pub fn res_n_reg(gb: &mut Gameboy, opcode: W<u8>) {
    let bitmask = !W(1u8.wrapping_shl((opcode >> 3).0 as u32 & 0x7));
    let lhs = instr_common::resolve_read_reg_low(gb, opcode);
    let value: W<u8> = lhs & bitmask;
    instr_common::resolve_write_reg_low(gb, opcode, value);
}

#[inline(always)]
pub fn sla_reg(gb: &mut Gameboy, opcode: W<u8>) {
    gb.reg.unset_all_flags();
    let lhs = instr_common::resolve_read_reg_low(gb, opcode);
    if lhs.0 & 0x80 != 0 {
        gb.reg.set_flag_c();
    } else {
        gb.reg.unset_flag_c();
    }
    
    let result = lhs << 1;

    if result.0 == 0 {
        gb.reg.set_flag_z();
    } else {
        gb.reg.unset_flag_z();
    }

    instr_common::resolve_write_reg_low(gb, opcode, result);
}

#[inline(always)]
pub fn sra_reg(gb: &mut Gameboy, opcode: W<u8>) {
    gb.reg.unset_all_flags();
    let lhs = instr_common::resolve_read_reg_low(gb, opcode);
    if lhs.0 & 0x1 != 0 {
        gb.reg.set_flag_c();
    } else {
        gb.reg.unset_flag_c();
    }

    let mut bit_0 = W(0u8);
    if lhs.0 & 0x80 != 0 {
        bit_0 = W(0x80u8);
    }
    
    let result = (lhs >> 1) | bit_0;

    if result.0 == 0 {
        gb.reg.set_flag_z();
    } else {
        gb.reg.unset_flag_z();
    }

    instr_common::resolve_write_reg_low(gb, opcode, result);
}

#[inline(always)]
pub fn srl_reg(gb: &mut Gameboy, opcode: W<u8>) {
    gb.reg.unset_all_flags();
    let lhs = instr_common::resolve_read_reg_low(gb, opcode);
    if lhs.0 & 0x1 != 0 {
        gb.reg.set_flag_c();
    } else {
        gb.reg.unset_flag_c();
    }

    let result = lhs >> 1;

    if result.0 == 0 {
        gb.reg.set_flag_z();
    } else {
        gb.reg.unset_flag_z();
    }

    instr_common::resolve_write_reg_low(gb, opcode, result);
}

#[inline(always)]
fn bit_n_reg(gb: &mut Gameboy, opcode: W<u8>) {
    let lhs = instr_common::resolve_read_reg_low(gb, opcode);
    let bitmask = W(1u8.wrapping_shl((opcode >> 3).0 as u32 & 0x7));

    if (lhs & bitmask).0 != 0 {
        gb.reg.unset_flag_z();
    } else {
        gb.reg.set_flag_z();
    }

    gb.reg.unset_flag_n();
    gb.reg.set_flag_h();
}

#[inline(always)]
fn rl_reg(gb: &mut Gameboy, opcode: W<u8>) {
    let lhs = instr_common::resolve_read_reg_low(gb, opcode);
    let old_carry = W(gb.reg.get_flag_c() as u8);
    let bit_7 = if (lhs.0 & 0x80) != 0 { true } else { false };
    let result = (lhs << 1) | old_carry;
    gb.reg.unset_all_flags();

    if bit_7 {
        gb.reg.set_flag_c();
    } else {
        gb.reg.unset_flag_c();
    }

    if result.0 == 0 {
        gb.reg.set_flag_z();
    } else {
        gb.reg.unset_flag_z();
    }

    instr_common::resolve_write_reg_low(gb, opcode, result);
}


#[inline(always)]
fn rr_reg(gb: &mut Gameboy, opcode: W<u8>) {
    let lhs = instr_common::resolve_read_reg_low(gb, opcode);
    let old_carry = if gb.reg.get_flag_c() { W(0x80u8) } else { W(0x00u8) };
    let bit_0 = if (lhs.0 & 0x1) != 0 { true } else { false };
    let result = (lhs >> 1) | old_carry;

    gb.reg.unset_all_flags();

    if bit_0 {
        gb.reg.set_flag_c();
    } else {
        gb.reg.unset_flag_c();
    }

    if result.0 == 0 {
        gb.reg.set_flag_z();
    } else {
        gb.reg.unset_flag_z();
    }

    instr_common::resolve_write_reg_low(gb, opcode, result);
}

#[inline(always)]
pub fn rlc_reg(gb: &mut Gameboy, opcode: W<u8>) {
    let lhs = instr_common::resolve_read_reg_low(gb, opcode);
    let bit_7 = (lhs & W(0x80u8)) >> 7;
    let result = (lhs << 1) | bit_7;
    gb.reg.unset_all_flags();

    if bit_7.0 != 0 {
        gb.reg.set_flag_c();
    } else {
        gb.reg.unset_flag_c();
    }

    if result.0 == 0 {
        gb.reg.set_flag_z();
    } else {
        gb.reg.unset_flag_z();
    }

    instr_common::resolve_write_reg_low(gb, opcode, result);
}

#[inline(always)]
fn rrc_reg(gb: &mut Gameboy, opcode: W<u8>) {
    let lhs = instr_common::resolve_read_reg_low(gb, opcode);
    let bit_0 = (lhs & W(0x1)) << 7;
    let result = (lhs >> 1) | bit_0;

    gb.reg.unset_all_flags();

    if bit_0.0 != 0 {
        gb.reg.set_flag_c();
    } else {
        gb.reg.unset_flag_c();
    }

    if result.0 == 0 {
        gb.reg.set_flag_z();
    } else {
        gb.reg.unset_flag_z();
    }

    instr_common::resolve_write_reg_low(gb, opcode, result);
}

#[inline(always)]
pub fn swap_reg(gb: &mut Gameboy, opcode: W<u8>) {
    let lhs = instr_common::resolve_read_reg_low(gb, opcode);
    let result = (lhs << 4) | (lhs >> 4);

    gb.reg.unset_all_flags();

    if result.0 == 0 {
        gb.reg.set_flag_z();
    } else {
        gb.reg.unset_flag_z();
    }

    instr_common::resolve_write_reg_low(gb, opcode, result);
}