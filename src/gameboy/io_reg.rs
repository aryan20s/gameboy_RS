use core::num::Wrapping as W;
use std::ops::BitAnd;

use super::{Gameboy, UNDEFINED_READ};

use log::debug;

const JOYPAD_IO: u8 = 0x00;
const COUNTER_DIV: u8 = 0x04;
const COUNTER_TIMA: u8 = 0x05;
const COUNTER_TMA: u8 = 0x06;
const COUNTER_TAC: u8 = 0x07;
const INT_FLAG: u8 = 0x0f;
const PPU_LCDC: u8 = 0x40;
const PPU_LCD_STAT: u8 = 0x41;
const PPU_SCROLL_Y: u8 = 0x42;
const PPU_SCROLL_X: u8 = 0x43;
const PPU_LCD_Y: u8 = 0x44;
const PPU_LY_COMPARE: u8 = 0x45;
const PPU_BGPAL: u8 = 0x47;
const PPU_WX: u8 = 0x4a;
const PPU_WY: u8 = 0x4b;
const BOOTROM_DISABLE: u8 = 0x50;

pub fn read_byte(gb: &mut Gameboy, addr: u8) -> u8 {
    match addr {
        JOYPAD_IO => {
            return gb.other_state.joypad_io_state;
        }
        COUNTER_DIV => {
            return (gb.other_state.counter_div >> 8).0 as u8;
        }
        INT_FLAG => {
            return 0b1110_0000 | gb.other_state.int_flag;
        }
        PPU_LCDC => {
            return gb.ppu.get_lcdc();
        }
        PPU_LCD_STAT => {
            return gb.ppu.get_stat();
        }
        PPU_SCROLL_Y => {
            return gb.ppu.scroll_y.0;
        }
        PPU_SCROLL_X => {
            return gb.ppu.scroll_x.0;
        }
        PPU_LCD_Y => {
            if gb.other_state.gb_doctor_mode && !gb.other_state.bootrom_enabled {
                return 0x90;
            }
            return gb.ppu.get_current_y();
        }
        PPU_LY_COMPARE => {
            return gb.ppu.get_ly_compare();
        }
        PPU_BGPAL => {
            return gb.ppu.get_bgpal();
        }
        PPU_WX => {
            return gb.ppu.window_x;
        }
        PPU_WY => {
            return gb.ppu.window_y;
        }
        BOOTROM_DISABLE => {
            return if gb.other_state.bootrom_enabled { 0 } else { 1 };
        }
        _ => {
            //debug!("Unimplemented IO read from {:#04x}", addr);
            return UNDEFINED_READ;
        }
    }
}

pub fn write_byte(gb: &mut Gameboy, addr: u8, value: u8) {
    match addr {
        JOYPAD_IO => {
            let mut new_val = 0xf | (value & 0x30);
            if (value & 0x20) == 0 {
                if gb.other_state.input_keys[0].get_held() {
                    new_val &= !0x8;
                }
                if gb.other_state.input_keys[1].get_held() {
                    new_val &= !0x4;
                }
                if gb.other_state.input_keys[2].get_held() {
                    new_val &= !0x2;
                }
                if gb.other_state.input_keys[3].get_held() {
                    new_val &= !0x1;
                }
            }

            if (value & 0x10) == 0 {
                if gb.other_state.input_keys[4].get_held() {
                    new_val &= !0x8;
                } else if gb.other_state.input_keys[5].get_held() {
                    new_val &= !0x4;
                }

                if gb.other_state.input_keys[6].get_held() {
                    new_val &= !0x2;
                } else if gb.other_state.input_keys[7].get_held() {
                    new_val &= !0x1;
                }
            }
            
            gb.other_state.joypad_io_state = new_val;
        }
        COUNTER_DIV => {
            gb.other_state.counter_div = W(0);
        }
        INT_FLAG => {
            gb.other_state.int_flag = value & 0x1f;
        }
        PPU_LCDC => {
            gb.ppu.set_lcdc(value);
        }
        PPU_LCD_STAT => {
            gb.ppu.set_stat(value);
        }
        PPU_SCROLL_Y => {
            gb.ppu.scroll_y = W(value);
        }
        PPU_SCROLL_X => {
            gb.ppu.scroll_x = W(value);
        }
        PPU_LCD_Y => {
            return;
        }
        PPU_LY_COMPARE => {
            gb.ppu.set_ly_compare(value);
        }
        PPU_BGPAL => {
            gb.ppu.set_bgpal(value);
        }
        PPU_WX => {
            gb.ppu.window_x = value;
        }
        PPU_WY => {
            gb.ppu.window_y = value;
        }
        BOOTROM_DISABLE => {
            if value & 0x1 == 0x1 {
                gb.other_state.bootrom_enabled = false;
                gb.other_state.instrs_run = 0;
            }
        }
        _ => {
            //debug!("Unimplemented IO write {:#04x} to {:#04x}", value, addr);
            return;
        }
    }
}
