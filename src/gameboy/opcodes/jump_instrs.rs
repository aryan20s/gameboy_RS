use crate::gameboy::Gameboy;
use core::num::Wrapping as W;

use log::debug;

#[inline(always)]
pub fn jr_nz_i8(gb: &mut Gameboy) {
    let offset = W(gb.read_byte_inc_pc().0 as i8 as u16);
    if !gb.reg.get_flag_z() {
        gb.pc += offset;
        gb.cycles_pending += 4;
    }
}

#[inline(always)]
pub fn jr_nc_i8(gb: &mut Gameboy) {
    let offset = W(gb.read_byte_inc_pc().0 as i8 as u16);
    if !gb.reg.get_flag_c() {
        gb.pc += offset;
        gb.cycles_pending += 4;
    }
}

#[inline(always)]
pub fn jr_i8(gb: &mut Gameboy) {
    let offset = W(gb.read_byte_inc_pc().0 as i8 as u16);
    gb.pc += offset;
    gb.cycles_pending += 4;
}

#[inline(always)]
pub fn jr_z_i8(gb: &mut Gameboy) {
    let offset = W(gb.read_byte_inc_pc().0 as i8 as u16);
    if gb.reg.get_flag_z() {
        gb.pc += offset;
        gb.cycles_pending += 4;
    }
}

#[inline(always)]
pub fn jr_c_i8(gb: &mut Gameboy) {
    let offset = W(gb.read_byte_inc_pc().0 as i8 as u16);
    if gb.reg.get_flag_c() {
        gb.pc += offset;
        gb.cycles_pending += 4;
    }
}

#[inline(always)]
pub fn call_u16(gb: &mut Gameboy) {
    let address = gb.read_short_inc_pc();
    gb.push_short(gb.pc);
    gb.pc = address;
    gb.cycles_pending += 4;
}

#[inline(always)]
pub fn call_nz_u16(gb: &mut Gameboy) {
    let address = gb.read_short_inc_pc();
    if !gb.reg.get_flag_z() {
        gb.push_short(gb.pc);
        gb.pc = address;
        gb.cycles_pending += 4;
    }
}

#[inline(always)]
pub fn call_nc_u16(gb: &mut Gameboy) {
    let address = gb.read_short_inc_pc();
    if !gb.reg.get_flag_c() {
        gb.push_short(gb.pc);
        gb.pc = address;
        gb.cycles_pending += 4;
    }
}

#[inline(always)]
pub fn call_z_u16(gb: &mut Gameboy) {
    let address = gb.read_short_inc_pc();
    if gb.reg.get_flag_z() {
        gb.push_short(gb.pc);
        gb.pc = address;
        gb.cycles_pending += 4;
    }
}

#[inline(always)]
pub fn call_c_u16(gb: &mut Gameboy) {
    let address = gb.read_short_inc_pc();
    if gb.reg.get_flag_c() {
        gb.push_short(gb.pc);
        gb.pc = address;
        gb.cycles_pending += 4;
    }
}

#[inline(always)]
pub fn ret(gb: &mut Gameboy) {
    gb.pc = gb.pop_short();
    gb.cycles_pending += 4;
}

#[inline(always)]
pub fn ret_nz(gb: &mut Gameboy) {
    gb.cycles_pending += 4;
    if !gb.reg.get_flag_z() {
        gb.pc = gb.pop_short();
    }
}

#[inline(always)]
pub fn ret_nc(gb: &mut Gameboy) {
    gb.cycles_pending += 4;
    if !gb.reg.get_flag_c() {
        gb.pc = gb.pop_short();
    }
}

#[inline(always)]
pub fn ret_z(gb: &mut Gameboy) {
    gb.cycles_pending += 4;
    if gb.reg.get_flag_z() {
        gb.pc = gb.pop_short();
    }
}

#[inline(always)]
pub fn ret_c(gb: &mut Gameboy) {
    gb.cycles_pending += 4;
    if gb.reg.get_flag_c() {
        gb.pc = gb.pop_short();
    }
}

#[inline(always)]
pub fn reti(gb: &mut Gameboy) {
    gb.pc = gb.pop_short();
    gb.ime = true;
    gb.cycles_pending += 4;
}

#[inline(always)]
pub fn rst_n(gb: &mut Gameboy, opcode: W<u8>) {
    let jump_addr = opcode.0 & 0x38;
    gb.push_short(gb.pc);
    gb.pc = W(jump_addr as u16);
    gb.cycles_pending += 4;
}

#[inline(always)]
pub fn jp_u16(gb: &mut Gameboy) {
    let jump_addr = gb.read_short_inc_pc();
    gb.cycles_pending += 4;
    gb.pc = jump_addr;
}

#[inline(always)]
pub fn jp_nz_u16(gb: &mut Gameboy) {
    let jump_addr = gb.read_short_inc_pc();
    if !gb.reg.get_flag_z() {
        gb.cycles_pending += 4;
        gb.pc = jump_addr;
    }
}

#[inline(always)]
pub fn jp_nc_u16(gb: &mut Gameboy) {
    let jump_addr = gb.read_short_inc_pc();
    if !gb.reg.get_flag_c() {
        gb.cycles_pending += 4;
        gb.pc = jump_addr;
    }
}

#[inline(always)]
pub fn jp_z_u16(gb: &mut Gameboy) {
    let jump_addr = gb.read_short_inc_pc();
    if gb.reg.get_flag_z() {
        gb.cycles_pending += 4;
        gb.pc = jump_addr;
    }
}

#[inline(always)]
pub fn jp_c_u16(gb: &mut Gameboy) {
    let jump_addr = gb.read_short_inc_pc();
    if gb.reg.get_flag_c() {
        gb.cycles_pending += 4;
        gb.pc = jump_addr;
    }
}

#[inline(always)]
pub fn jp_hl(gb: &mut Gameboy) {
    gb.pc = gb.reg.get_hl();
}
