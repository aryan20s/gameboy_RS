use crate::gameboy::Gameboy;
use core::num::Wrapping as W;

use super::instr_common;

const FLAG_Z: W<u8> = W(0x80);
const FLAG_H: W<u8> = W(0x20);

#[inline(always)]
pub fn set_op_flags_add(gb: &mut Gameboy, lhs: W<u8>, rhs: W<u8>, carry: bool, zero: bool) {
    if (lhs.0 & 0xF) + (rhs.0 & 0xF) >= 0x10 {
        gb.reg.set_flag_h();
    } else {
        gb.reg.unset_flag_h();
    }

    if carry {
        if ((lhs.0 as u16) + (rhs.0 as u16)) > 0xff {
            gb.reg.set_flag_c();
        } else {
            gb.reg.unset_flag_c();
        }
    }

    if zero {
        if (lhs + rhs).0 == 0 {
            gb.reg.set_flag_z();
        } else {
            gb.reg.unset_flag_z();
        }
    }
}

#[inline(always)]
pub fn set_op_flags_adc(gb: &mut Gameboy, lhs: W<u8>, rhs: W<u8>, carry: bool, zero: bool) {
    let carry_flag_set = W(if gb.reg.get_flag_c() { 1u8 } else { 0u8 });

    if (lhs.0 & 0xF) + (rhs.0 & 0xF) + carry_flag_set.0 >= 0x10 {
        gb.reg.set_flag_h();
    } else {
        gb.reg.unset_flag_h();
    }

    if carry {
        if (lhs.0 as u16) + (rhs.0 as u16) + (carry_flag_set.0 as u16) >= 0x100 {
            gb.reg.set_flag_c();
        } else {
            gb.reg.unset_flag_c();
        }
    }

    if zero {
        if (lhs + rhs + carry_flag_set).0 == 0 {
            gb.reg.set_flag_z();
        } else {
            gb.reg.unset_flag_z();
        }
    }
}

#[inline(always)]
pub fn set_op_flags_sub(gb: &mut Gameboy, lhs: W<u8>, rhs: W<u8>, carry: bool, zero: bool) {
    if (lhs & W(0xf)) < (rhs & W(0xf)) {
        gb.reg.set_flag_h();
    } else {
        gb.reg.unset_flag_h();
    }

    if carry {
        if lhs < rhs {
            gb.reg.set_flag_c();
        } else {
            gb.reg.unset_flag_c();
        }
    }

    if zero {
        if (lhs - rhs).0 == 0 {
            gb.reg.set_flag_z();
        } else {
            gb.reg.unset_flag_z();
        }
    }
}

#[inline(always)]
pub fn set_op_flags_sbc(gb: &mut Gameboy, lhs: W<u8>, rhs: W<u8>, carry: bool, zero: bool) {
    let carry_flag_set = W(if gb.reg.get_flag_c() { 1u8 } else { 0u8 });

    if (lhs & W(0xf)) < ((rhs & W(0xf)) + carry_flag_set) {
        gb.reg.set_flag_h();
    } else {
        gb.reg.unset_flag_h();
    }

    if carry {
        if (lhs.0 as u16) < (rhs.0 as u16).wrapping_add(carry_flag_set.0 as u16) {
            gb.reg.set_flag_c();
        } else {
            gb.reg.unset_flag_c();
        }
    }

    if zero {
        if (lhs - rhs - carry_flag_set).0 == 0 {
            gb.reg.set_flag_z();
        } else {
            gb.reg.unset_flag_z();
        }
    }
}

#[inline(always)]
pub fn add_a_reg(gb: &mut Gameboy, opcode: W<u8>) {
    let rhs = instr_common::resolve_read_reg_low(gb, opcode);
    set_op_flags_add(gb, gb.reg.a, rhs, true, true);
    gb.reg.unset_flag_n();
    gb.reg.a += rhs;
}

#[inline(always)]
pub fn adc_a_reg(gb: &mut Gameboy, opcode: W<u8>) {
    let rhs = instr_common::resolve_read_reg_low(gb, opcode);
    let flag_c_set = W(if gb.reg.get_flag_c() { 1 } else { 0 });
    set_op_flags_adc(gb, gb.reg.a, rhs, true, true);
    gb.reg.unset_flag_n();
    gb.reg.a += rhs + flag_c_set;
}

#[inline(always)]
pub fn sub_a_reg(gb: &mut Gameboy, opcode: W<u8>) {
    let rhs = instr_common::resolve_read_reg_low(gb, opcode);
    set_op_flags_sub(gb, gb.reg.a, rhs, true, true);
    gb.reg.set_flag_n();
    gb.reg.a -= rhs;
}

#[inline(always)]
pub fn sbc_a_reg(gb: &mut Gameboy, opcode: W<u8>) {
    let rhs = instr_common::resolve_read_reg_low(gb, opcode);
    let flag_c_set = W(if gb.reg.get_flag_c() { 1 } else { 0 });
    set_op_flags_sbc(gb, gb.reg.a, rhs, true, true);
    gb.reg.set_flag_n();
    gb.reg.a -= rhs + flag_c_set;
}

#[inline(always)]
pub fn and_a_reg(gb: &mut Gameboy, opcode: W<u8>) {
    let rhs = instr_common::resolve_read_reg_low(gb, opcode);
    gb.reg.a &= rhs;

    gb.reg.unset_all_flags();
    gb.reg
        .set_f(FLAG_H | if gb.reg.a == W(0) { FLAG_Z } else { W(0) });
}

#[inline(always)]
pub fn xor_a_reg(gb: &mut Gameboy, opcode: W<u8>) {
    let rhs = instr_common::resolve_read_reg_low(gb, opcode);
    gb.reg.a ^= rhs;

    if gb.reg.a.0 == 0 {
        gb.reg.set_f(FLAG_Z);
    } else {
        gb.reg.unset_all_flags();
    }
}

#[inline(always)]
pub fn or_a_reg(gb: &mut Gameboy, opcode: W<u8>) {
    let rhs = instr_common::resolve_read_reg_low(gb, opcode);
    gb.reg.a |= rhs;

    if gb.reg.a.0 == 0 {
        gb.reg.set_f(FLAG_Z);
    } else {
        gb.reg.unset_all_flags();
    }
}

#[inline(always)]
pub fn cp_a_reg(gb: &mut Gameboy, opcode: W<u8>) {
    let rhs = instr_common::resolve_read_reg_low(gb, opcode);
    set_op_flags_sub(gb, gb.reg.a, rhs, true, true);
    gb.reg.set_flag_n();
}

#[inline(always)]
pub fn inc_reg(gb: &mut Gameboy, opcode: W<u8>) {
    gb.reg.unset_flag_n();
    let lhs = instr_common::resolve_read_reg_high(gb, opcode);
    let rhs = W(1u8);
    set_op_flags_add(gb, lhs, rhs, false, true);
    instr_common::resolve_write_reg_high(gb, opcode, lhs + rhs);
}

#[inline(always)]
pub fn dec_reg(gb: &mut Gameboy, opcode: W<u8>) {
    gb.reg.set_flag_n();
    let lhs = instr_common::resolve_read_reg_high(gb, opcode);
    let rhs = W(-1i8 as u8);
    set_op_flags_sub(gb, lhs, W(1), false, true);
    instr_common::resolve_write_reg_high(gb, opcode, lhs + rhs);
}

#[inline(always)]
pub fn inc_bc(gb: &mut Gameboy) {
    let mut value = gb.reg.get_bc();
    value += 1;
    gb.reg.set_bc(value);
    gb.cycles_pending += 4;
}

#[inline(always)]
pub fn inc_de(gb: &mut Gameboy) {
    let mut value = gb.reg.get_de();
    value += 1;
    gb.reg.set_de(value);
    gb.cycles_pending += 4;
}

#[inline(always)]
pub fn inc_hl(gb: &mut Gameboy) {
    let mut value = gb.reg.get_hl();
    value += 1;
    gb.reg.set_hl(value);
    gb.cycles_pending += 4;
}

#[inline(always)]
pub fn inc_sp(gb: &mut Gameboy) {
    gb.sp += 1;
    gb.cycles_pending += 4;
}

#[inline(always)]
pub fn dec_bc(gb: &mut Gameboy) {
    let mut value = gb.reg.get_bc();
    value -= 1;
    gb.reg.set_bc(value);
    gb.cycles_pending += 4;
}

#[inline(always)]
pub fn dec_de(gb: &mut Gameboy) {
    let mut value = gb.reg.get_de();
    value -= 1;
    gb.reg.set_de(value);
    gb.cycles_pending += 4;
}

#[inline(always)]
pub fn dec_hl(gb: &mut Gameboy) {
    let mut value = gb.reg.get_hl();
    value -= 1;
    gb.reg.set_hl(value);
    gb.cycles_pending += 4;
}

#[inline(always)]
pub fn dec_sp(gb: &mut Gameboy) {
    gb.sp -= 1;
    gb.cycles_pending += 4;
}

#[inline(always)]
pub fn add_a_u8(gb: &mut Gameboy) {
    let rhs = gb.read_byte_inc_pc();
    set_op_flags_add(gb, gb.reg.a, rhs, true, true);
    gb.reg.unset_flag_n();
    gb.reg.a += rhs;
}

#[inline(always)]
pub fn adc_a_u8(gb: &mut Gameboy) {
    let rhs = gb.read_byte_inc_pc();
    let flag_c_set = W(if gb.reg.get_flag_c() { 1 } else { 0 });
    set_op_flags_adc(gb, gb.reg.a, rhs, true, true);
    gb.reg.unset_flag_n();
    gb.reg.a += rhs + flag_c_set;
}

#[inline(always)]
pub fn sub_a_u8(gb: &mut Gameboy) {
    let rhs = gb.read_byte_inc_pc();
    set_op_flags_sub(gb, gb.reg.a, rhs, true, true);
    gb.reg.set_flag_n();
    gb.reg.a -= rhs;
}

#[inline(always)]
pub fn sbc_a_u8(gb: &mut Gameboy) {
    let rhs = gb.read_byte_inc_pc();
    let flag_c_set = W(if gb.reg.get_flag_c() { 1 } else { 0 });
    set_op_flags_sbc(gb, gb.reg.a, rhs, true, true);
    gb.reg.set_flag_n();
    gb.reg.a -= rhs + flag_c_set;
}

#[inline(always)]
pub fn and_a_u8(gb: &mut Gameboy) {
    let rhs = gb.read_byte_inc_pc();
    gb.reg.a &= rhs;

    gb.reg.unset_all_flags();
    gb.reg
        .set_f(FLAG_H | if gb.reg.a == W(0) { FLAG_Z } else { W(0) });
}

#[inline(always)]
pub fn xor_a_u8(gb: &mut Gameboy) {
    let rhs = gb.read_byte_inc_pc();
    gb.reg.a ^= rhs;

    if gb.reg.a.0 == 0 {
        gb.reg.set_f(FLAG_Z);
    } else {
        gb.reg.unset_all_flags();
    }
}

#[inline(always)]
pub fn or_a_u8(gb: &mut Gameboy) {
    let rhs = gb.read_byte_inc_pc();
    gb.reg.a |= rhs;

    if gb.reg.a.0 == 0 {
        gb.reg.set_f(FLAG_Z);
    } else {
        gb.reg.unset_all_flags();
    }
}

#[inline(always)]
pub fn cp_a_u8(gb: &mut Gameboy) {
    let rhs = gb.read_byte_inc_pc();
    set_op_flags_sub(gb, gb.reg.a, rhs, true, true);
    gb.reg.set_flag_n();
}

#[inline(always)]
pub fn add_hl_bc(gb: &mut Gameboy) {
    let mut lhs = gb.reg.h;
    if (gb.reg.l.0 as u16 + gb.reg.c.0 as u16) >= 0x100 {
        lhs += W(1);
    }
    set_op_flags_add(gb, lhs, gb.reg.b, true, false);
    let lhs = gb.reg.get_hl();
    let rhs = gb.reg.get_bc();
    gb.reg.set_hl(rhs + lhs);
    gb.reg.unset_flag_n();
    gb.cycles_pending += 4;
}

#[inline(always)]
pub fn add_hl_de(gb: &mut Gameboy) {
    let lhs = gb.reg.h;
    set_op_flags_add(gb, lhs, gb.reg.d, true, false);
    let lhs = gb.reg.get_hl();
    let rhs = gb.reg.get_de();
    gb.reg.set_hl(rhs + lhs);
    gb.reg.unset_flag_n();
    gb.cycles_pending += 4;
}

#[inline(always)]
pub fn add_hl_hl(gb: &mut Gameboy) {
    let lhs = gb.reg.h;
    set_op_flags_add(gb, lhs, gb.reg.h, true, false);
    let lhs = gb.reg.get_hl();
    let rhs = gb.reg.get_hl();
    gb.reg.set_hl(rhs + lhs);
    gb.reg.unset_flag_n();
    gb.cycles_pending += 4;
}

#[inline(always)]
pub fn add_hl_sp(gb: &mut Gameboy) {
    let lhs = gb.reg.h;
    set_op_flags_add(gb, lhs, W((gb.sp >> 8).0 as u8), true, false);
    let lhs = gb.reg.get_hl();
    let rhs = gb.sp;
    gb.reg.set_hl(rhs + lhs);
    gb.reg.unset_flag_n();
    gb.cycles_pending += 4;
}

#[inline(always)]
pub fn rla(gb: &mut Gameboy) {
    let lhs = gb.reg.a;
    let old_carry = W(gb.reg.get_flag_c() as u8);
    let bit_7 = if (lhs.0 & 0x80) != 0 { true } else { false };
    let result = (lhs << 1) | old_carry;
    gb.reg.unset_all_flags();

    if bit_7 {
        gb.reg.set_flag_c();
    } else {
        gb.reg.unset_flag_c();
    }

    gb.reg.a = result;
}

#[inline(always)]
pub fn rra(gb: &mut Gameboy) {
    let lhs = gb.reg.a;
    let old_carry = if gb.reg.get_flag_c() {
        W(0x80u8)
    } else {
        W(0x00u8)
    };
    let bit_0 = if (lhs.0 & 0x1) != 0 { true } else { false };
    let result = (lhs >> 1) | old_carry;
    gb.reg.unset_all_flags();

    if bit_0 {
        gb.reg.set_flag_c();
    } else {
        gb.reg.unset_flag_c();
    }

    gb.reg.a = result;
}

#[inline(always)]
pub fn rlca(gb: &mut Gameboy) {
    let lhs = gb.reg.a;
    let bit_7 = (lhs & W(0x80u8)) >> 7;
    let result = (lhs << 1) | bit_7;
    gb.reg.unset_all_flags();

    if bit_7.0 != 0 {
        gb.reg.set_flag_c();
    } else {
        gb.reg.unset_flag_c();
    }

    gb.reg.a = result;
}

#[inline(always)]
pub fn rrca(gb: &mut Gameboy) {
    let lhs = gb.reg.a;
    let bit_0 = (lhs & W(0x1)) << 7;
    let result = (lhs >> 1) | bit_0;
    gb.reg.unset_all_flags();

    if bit_0.0 != 0 {
        gb.reg.set_flag_c();
    } else {
        gb.reg.unset_flag_c();
    }

    gb.reg.a = result;
}

#[inline(always)]
pub fn cpl(gb: &mut Gameboy) {
    gb.reg.a = !gb.reg.a;
    gb.reg.set_flag_n();
    gb.reg.set_flag_h();
}

#[inline(always)]
pub fn add_sp_i8(gb: &mut Gameboy) {
    let rhs = W(gb.read_byte_inc_pc().0 as i8 as u16);
    let lhs = gb.sp;
    gb.reg.unset_all_flags();
    set_op_flags_add(gb, W(lhs.0 as u8), W(rhs.0 as u8), true, false);
    gb.sp = lhs + rhs;
    gb.cycles_pending += 8;
}

#[inline(always)]
pub fn ld_hl_sp_i8(gb: &mut Gameboy) {
    let rhs = W(gb.read_byte_inc_pc().0 as i8 as u16);
    let lhs = gb.sp;
    gb.reg.unset_all_flags();
    set_op_flags_add(gb, W(lhs.0 as u8), W(rhs.0 as u8), true, false);
    gb.reg.set_hl(lhs + rhs);
}

pub fn daa(gb: &mut Gameboy) {
    if !gb.reg.get_flag_n() {
        if gb.reg.get_flag_c() || gb.reg.a.0 > 0x99 {
            gb.reg.a += W(0x60);
            gb.reg.set_flag_c();
        }
        if gb.reg.get_flag_h() || (gb.reg.a.0 & 0x0f) > 0x09 {
            gb.reg.a += W(0x6);
        }
    } else {
        if gb.reg.get_flag_c() {
            gb.reg.a -= W(0x60);
        }
        if gb.reg.get_flag_h() {
            gb.reg.a -= W(0x6);
        }
    }

    if gb.reg.a.0 == 0 {
        gb.reg.set_flag_z();
    } else {
        gb.reg.unset_flag_z();
    }

    gb.reg.unset_flag_h();
}
