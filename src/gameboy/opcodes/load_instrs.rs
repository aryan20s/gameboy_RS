use log::debug;

use crate::gameboy::Gameboy;
use core::num::Wrapping as W;

use super::instr_common;

#[inline(always)]
pub fn ld_bc_u16(gb: &mut Gameboy) {
    let val = gb.read_short_inc_pc();
    gb.reg.set_bc(val);
}

#[inline(always)]
pub fn ld_de_u16(gb: &mut Gameboy) {
    let val = gb.read_short_inc_pc();
    gb.reg.set_de(val);
}

#[inline(always)]
pub fn ld_hl_u16(gb: &mut Gameboy) {
    let val = gb.read_short_inc_pc();
    gb.reg.set_hl(val);
}

#[inline(always)]
pub fn ld_sp_u16(gb: &mut Gameboy) {
    gb.sp = gb.read_short_inc_pc();
}

#[inline(always)]
pub fn ld_reg_reg(gb: &mut Gameboy, opcode: W<u8>) {
    let value = instr_common::resolve_read_reg_low(gb, opcode);
    instr_common::resolve_write_reg_high(gb, opcode, value);
}

#[inline(always)]
pub fn ld_addr_bc_a(gb: &mut Gameboy) {
    let addr = gb.reg.get_bc();
    gb.write_byte(addr, gb.reg.a);
}

#[inline(always)]
pub fn ld_addr_de_a(gb: &mut Gameboy) {
    let addr = gb.reg.get_de();
    gb.write_byte(addr, gb.reg.a);
}

#[inline(always)]
pub fn ld_addr_inc_hl_a(gb: &mut Gameboy) {
    let addr = gb.reg.get_hl();
    gb.write_byte(addr, gb.reg.a);
    gb.reg.set_hl(addr + W(1));
}

#[inline(always)]
pub fn ld_addr_dec_hl_a(gb: &mut Gameboy) {
    let addr = gb.reg.get_hl();
    gb.write_byte(addr, gb.reg.a);
    gb.reg.set_hl(addr - W(1));
}

#[inline(always)]
pub fn ld_reg_u8(gb: &mut Gameboy, opcode: W<u8>) {
    let value = gb.read_byte_inc_pc();
    instr_common::resolve_write_reg_high(gb, opcode, value);
}

#[inline(always)]
pub fn ld_a_addr_bc(gb: &mut Gameboy) {
    let addr = gb.reg.get_bc();
    gb.reg.a = gb.read_byte(addr);
}

#[inline(always)]
pub fn ld_a_addr_de(gb: &mut Gameboy) {
    let addr = gb.reg.get_de();
    gb.reg.a = gb.read_byte(addr);
}

#[inline(always)]
pub fn ld_a_addr_inc_hl(gb: &mut Gameboy) {
    let addr = gb.reg.get_hl();
    gb.reg.a = gb.read_byte(addr);
    gb.reg.set_hl(addr + W(1));
}

#[inline(always)]
pub fn ld_a_addr_dec_hl(gb: &mut Gameboy) {
    let addr = gb.reg.get_hl();
    gb.reg.a = gb.read_byte(addr);
    gb.reg.set_hl(addr - W(1));
}

#[inline(always)]
pub fn ld_ff00_u8_a(gb: &mut Gameboy) {
    let offset = W(gb.read_byte_inc_pc().0 as u16);
    gb.write_byte(W(0xff00u16) + offset, gb.reg.a);
}

#[inline(always)]
pub fn ld_a_ff00_u8(gb: &mut Gameboy) {
    let offset = W(gb.read_byte_inc_pc().0 as u16);
    gb.reg.a = gb.read_byte(W(0xff00u16) + offset);
}

#[inline(always)]
pub fn ld_ff00_c_a(gb: &mut Gameboy) {
    gb.write_byte(W(0xff00u16) + W(gb.reg.c.0 as u16), gb.reg.a);
}

#[inline(always)]
pub fn ld_a_ff00_c(gb: &mut Gameboy) {
    gb.reg.a = gb.read_byte(W(0xff00u16) + W(gb.reg.c.0 as u16));
}

#[inline(always)]
pub fn push_bc(gb: &mut Gameboy) {
    gb.push_short(gb.reg.get_bc());
}

#[inline(always)]
pub fn push_de(gb: &mut Gameboy) {
    gb.push_short(gb.reg.get_de());
}

#[inline(always)]
pub fn push_hl(gb: &mut Gameboy) {
    gb.push_short(gb.reg.get_hl());
}

#[inline(always)]
pub fn push_af(gb: &mut Gameboy) {
    gb.push_short(gb.reg.get_af());
}

#[inline(always)]
pub fn pop_bc(gb: &mut Gameboy) {
    let value = gb.pop_short();
    gb.reg.set_bc(value);
}

#[inline(always)]
pub fn pop_de(gb: &mut Gameboy) {
    let value = gb.pop_short();
    gb.reg.set_de(value);
}

#[inline(always)]
pub fn pop_hl(gb: &mut Gameboy) {
    let value = gb.pop_short();
    gb.reg.set_hl(value);
}

#[inline(always)]
pub fn pop_af(gb: &mut Gameboy) {
    let value = gb.pop_short();
    gb.reg.set_af(value);
}

#[inline(always)]
pub fn ld_addr_u16_a(gb: &mut Gameboy) {
    let addr = gb.read_short_inc_pc();
    gb.write_byte(addr, gb.reg.a);
}

#[inline(always)]
pub fn ld_a_addr_u16(gb: &mut Gameboy) {
    let addr = gb.read_short_inc_pc();
    gb.reg.a = gb.read_byte(addr);
}

#[inline(always)]
pub fn ld_addr_u16_sp(gb: &mut Gameboy) {
    let addr = gb.read_short_inc_pc();
    gb.write_short(addr, gb.sp);
}

#[inline(always)]
pub fn ld_sp_hl(gb: &mut Gameboy) {
    gb.sp = gb.reg.get_hl();
}
