use super::Gameboy;
use core::num::Wrapping as W;

pub mod instr_common;
pub mod jump_instrs;
pub mod load_instrs;
pub mod math_instrs;
pub mod prefix_cb;
pub mod misc_instrs;

#[inline(always)]
pub fn run_opcode(gb: &mut Gameboy, opcode: W<u8>) -> bool {
    match opcode.0 {
        0x00 => {
            //nop
        }
        0x01 => {
            load_instrs::ld_bc_u16(gb);
        }
        0x02 => {
            load_instrs::ld_addr_bc_a(gb);
        }
        0x03 => {
            math_instrs::inc_bc(gb);
        }
        0x04 => {
            math_instrs::inc_reg(gb, opcode);
        }
        0x05 => {
            math_instrs::dec_reg(gb, opcode);
        }
        0x06 => {
            load_instrs::ld_reg_u8(gb, opcode);
        }
        0x07 => {
            math_instrs::rlca(gb);
        }
        0x08 => {
            load_instrs::ld_addr_u16_sp(gb);
        }
        0x09 => {
            math_instrs::add_hl_bc(gb);
        }
        0x0a => {
            load_instrs::ld_a_addr_bc(gb);
        }
        0x0b => {
            math_instrs::dec_bc(gb);
        }
        0x0c => {
            math_instrs::inc_reg(gb, opcode);
        }
        0x0d => {
            math_instrs::dec_reg(gb, opcode);
        }
        0x0e => {
            load_instrs::ld_reg_u8(gb, opcode);
        }
        0x0f => {
            math_instrs::rrca(gb);
        }
        0x11 => {
            load_instrs::ld_de_u16(gb);
        }
        0x12 => {
            load_instrs::ld_addr_de_a(gb);
        }
        0x13 => {
            math_instrs::inc_de(gb);
        }
        0x14 => {
            math_instrs::inc_reg(gb, opcode);
        }
        0x15 => {
            math_instrs::dec_reg(gb, opcode);
        }
        0x16 => {
            load_instrs::ld_reg_u8(gb, opcode);
        }
        0x17 => {
            math_instrs::rla(gb);
        }
        0x18 => {
            jump_instrs::jr_i8(gb);
        }
        0x19 => {
            math_instrs::add_hl_de(gb);
        }
        0x1a => {
            load_instrs::ld_a_addr_de(gb);
        }
        0x1b => {
            math_instrs::dec_de(gb);
        }
        0x1c => {
            math_instrs::inc_reg(gb, opcode);
        }
        0x1d => {
            math_instrs::dec_reg(gb, opcode);
        }
        0x1e => {
            load_instrs::ld_reg_u8(gb, opcode);
        }
        0x1f => {
            math_instrs::rra(gb);
        }
        0x20 => {
            jump_instrs::jr_nz_i8(gb);
        }
        0x21 => {
            load_instrs::ld_hl_u16(gb);
        }
        0x22 => {
            load_instrs::ld_addr_inc_hl_a(gb);
        }
        0x23 => {
            math_instrs::inc_hl(gb);
        }
        0x24 => {
            math_instrs::inc_reg(gb, opcode);
        }
        0x25 => {
            math_instrs::dec_reg(gb, opcode);
        }
        0x26 => {
            load_instrs::ld_reg_u8(gb, opcode);
        }
        0x27 => {
            math_instrs::daa(gb);
        }
        0x28 => {
            jump_instrs::jr_z_i8(gb);
        }
        0x29 => {
            math_instrs::add_hl_hl(gb);
        }
        0x2a => {
            load_instrs::ld_a_addr_inc_hl(gb);
        }
        0x2b => {
            math_instrs::dec_hl(gb);
        }
        0x2c => {
            math_instrs::inc_reg(gb, opcode);
        }
        0x2d => {
            math_instrs::dec_reg(gb, opcode);
        }
        0x2e => {
            load_instrs::ld_reg_u8(gb, opcode);
        }
        0x2f => {
            math_instrs::cpl(gb);
        }
        0x30 => {
            jump_instrs::jr_nc_i8(gb);
        }
        0x31 => {
            load_instrs::ld_sp_u16(gb);
        }
        0x32 => {
            load_instrs::ld_addr_dec_hl_a(gb);
        }
        0x33 => {
            math_instrs::inc_sp(gb);
        }
        0x34 => {
            math_instrs::inc_reg(gb, opcode);
        }
        0x35 => {
            math_instrs::dec_reg(gb, opcode);
        }
        0x36 => {
            load_instrs::ld_reg_u8(gb, opcode);
        }
        0x37 => {
            misc_instrs::scf(gb);
        }
        0x38 => {
            jump_instrs::jr_c_i8(gb);
        }
        0x39 => {
            math_instrs::add_hl_sp(gb);
        }
        0x3a => {
            load_instrs::ld_a_addr_dec_hl(gb);
        }
        0x3b => {
            math_instrs::dec_sp(gb);
        }
        0x3c => {
            math_instrs::inc_reg(gb, opcode);
        }
        0x3d => {
            math_instrs::dec_reg(gb, opcode);
        }
        0x3e => {
            load_instrs::ld_reg_u8(gb, opcode);
        }
        0x3f => {
            misc_instrs::ccf(gb);
        }
        0x40..=0x75 => {
            load_instrs::ld_reg_reg(gb, opcode);
        }
        0x76 => {
            misc_instrs::halt(gb);
        }
        0x77..=0x7f => {
            load_instrs::ld_reg_reg(gb, opcode);
        }
        0x80..=0x87 => {
            math_instrs::add_a_reg(gb, opcode);
        }
        0x88..=0x8f => {
            math_instrs::adc_a_reg(gb, opcode);
        }
        0x90..=0x97 => {
            math_instrs::sub_a_reg(gb, opcode);
        }
        0x98..=0x9f => {
            math_instrs::sbc_a_reg(gb, opcode);
        }
        0xa0..=0xa7 => {
            math_instrs::and_a_reg(gb, opcode);
        }
        0xa8..=0xaf => {
            math_instrs::xor_a_reg(gb, opcode);
        }
        0xb0..=0xb7 => {
            math_instrs::or_a_reg(gb, opcode);
        }
        0xb8..=0xbf => {
            math_instrs::cp_a_reg(gb, opcode);
        }
        0xc0 => {
            jump_instrs::ret_nz(gb);
        }
        0xc1 => {
            load_instrs::pop_bc(gb);
        }
        0xc2 => {
            jump_instrs::jp_nz_u16(gb);
        }
        0xc3 => {
            jump_instrs::jp_u16(gb);
        }
        0xc4 => {
            jump_instrs::call_nz_u16(gb);
        }
        0xc5 => {
            load_instrs::push_bc(gb);
        }
        0xc6 => {
            math_instrs::add_a_u8(gb);
        }
        0xc7 => {
            jump_instrs::rst_n(gb, opcode);
        }
        0xc8 => {
            jump_instrs::ret_z(gb);
        }
        0xc9 => {
            jump_instrs::ret(gb);
        }
        0xca => {
            jump_instrs::jp_z_u16(gb);
        }
        0xcb => {
            return prefix_cb::prefix_cb(gb);
        }
        0xcc => {
            jump_instrs::call_z_u16(gb);
        }
        0xcd => {
            jump_instrs::call_u16(gb);
        }
        0xce => {
            math_instrs::adc_a_u8(gb);
        }
        0xcf => {
            jump_instrs::rst_n(gb, opcode);
        }
        0xd0 => {
            jump_instrs::ret_nc(gb);
        }
        0xd1 => {
            load_instrs::pop_de(gb);
        }
        0xd2 => {
            jump_instrs::jp_nc_u16(gb);
        }
        0xd4 => {
            jump_instrs::call_nc_u16(gb);
        }
        0xd5 => {
            load_instrs::push_de(gb);
        }
        0xd6 => {
            math_instrs::sub_a_u8(gb);
        }
        0xd7 => {
            jump_instrs::rst_n(gb, opcode);
        }
        0xd8 => {
            jump_instrs::ret_c(gb);
        }
        0xd9 => {
            jump_instrs::reti(gb);
        }
        0xda => {
            jump_instrs::jp_c_u16(gb);
        }
        0xdc => {
            jump_instrs::call_c_u16(gb);
        }
        0xde => {
            math_instrs::sbc_a_u8(gb);
        }
        0xdf => {
            jump_instrs::rst_n(gb, opcode);
        }
        0xe0 => {
            load_instrs::ld_ff00_u8_a(gb);
        }
        0xe1 => {
            load_instrs::pop_hl(gb);
        }
        0xe2 => {
            load_instrs::ld_ff00_c_a(gb);
        }
        0xe5 => {
            load_instrs::push_hl(gb);
        }
        0xe6 => {
            math_instrs::and_a_u8(gb);
        }
        0xe7 => {
            jump_instrs::rst_n(gb, opcode);
        }
        0xe8 => {
            math_instrs::add_sp_i8(gb);
        }
        0xe9 => {
            jump_instrs::jp_hl(gb);
        }
        0xea => {
            load_instrs::ld_addr_u16_a(gb);
        }
        0xee => {
            math_instrs::xor_a_u8(gb);
        }
        0xef => {
            jump_instrs::rst_n(gb, opcode);
        }
        0xf0 => {
            load_instrs::ld_a_ff00_u8(gb);
        }
        0xf1 => {
            load_instrs::pop_af(gb);
        }
        0xf2 => {
            load_instrs::ld_a_ff00_c(gb);
        }
        0xf3 => {
            misc_instrs::di(gb);
        }
        0xf5 => {
            load_instrs::push_af(gb);
        }
        0xf6 => {
            math_instrs::or_a_u8(gb);
        }
        0xf7 => {
            jump_instrs::rst_n(gb, opcode);
        }
        0xf8 => {
            math_instrs::ld_hl_sp_i8(gb);
        }
        0xf9 => {
            load_instrs::ld_sp_hl(gb);
        }
        0xfa => {
            load_instrs::ld_a_addr_u16(gb);
        }
        0xfb => {
            misc_instrs::ei(gb);
        }
        0xfe => {
            math_instrs::cp_a_u8(gb);
        }
        0xff => {
            jump_instrs::rst_n(gb, opcode);
        }
        _ => {
            return false;
        }
    }
    return true;
}
