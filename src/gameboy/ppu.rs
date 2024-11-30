use core::num::Wrapping as W;

use super::{OtherState, SystemType, INT_STAT, INT_VBLANK};
use super::banked_memory::BankedMemory;

use log::debug;

#[derive(Clone, Copy, Debug)]
pub enum Color {
    Black = 0x00000000,
    DGray = 0x00555555,
    LGray = 0x00AAAAAA,
    White = 0x00FFFFFF
}

#[derive(PartialEq, Clone, Copy)]
enum PPUMode {
    HBlank = 0,
    VBlank = 1,
    OAMScan = 2,
    PixelPut = 3,
}

const COLORS: [Color; 4] = [Color::White, Color::LGray, Color::DGray, Color::Black];
const GB_SCREEN_WIDTH: usize = 160;
const GB_SCREEN_HEIGHT: usize = 144;
const VBLANK_LINES: usize = 10;
const OAM_SCAN_DOTS: u16 = 80;
const PIXEL_PUT_MIN_DOTS: u16 = 172;
const HBLANK_MAX_DOTS: u16 = 204;
const LINE_TOTAL_DOTS: u16 = OAM_SCAN_DOTS + PIXEL_PUT_MIN_DOTS + HBLANK_MAX_DOTS;

const LY_STAT_INT: u8 = 0x40;
const OAM_STAT_INT: u8 = 0x20;
const VBLANK_STAT_INT: u8 = 0x10;
const HBLANK_STAT_INT: u8 = 0x8;

pub struct PPU {
    vram: BankedMemory,
    oam: Vec<u8>,
    screen: Vec<u32>,

    ppu_enabled: bool, //lcdc
    window_tilemap_offset: bool,
    window_enable: bool,
    bg_window_tiledata_offset: bool,
    bg_tilemap_offset: bool,
    obj_size_is_8x16: bool,
    obj_enable: bool,
    bg_window_priority: bool,

    pub scroll_x: W<u8>, //scx
    pub scroll_y: W<u8>, //scy

    current_x: W<u8>,
    current_y: W<u8>, //ly
    current_window_y: W<u8>,

    bg_colors: [Color; 4], //bgpal
    obp1_colors: [Color; 4], //obp1
    obp2_colors: [Color; 4], //obp2

    current_mode_cycles: u64,
    mode_3_extra_dots: u16,
    current_mode: PPUMode,

    ly_stat_int: bool, //lcd stat
    oam_stat_int: bool,
    hblank_stat_int: bool,
    vblank_stat_int: bool,

    ly_compare: u8, //lyc

    pub dbg_tiledata_bg_swap: bool,
    pub dbg_tilemap_bg_swap: bool,
    pub dbg_tiledata_win_swap: bool,
    pub dbg_tilemap_win_swap: bool,
    pub dbg_win_toggle: bool,

    pub window_x: u8, //wx
    pub window_y: u8, //wy
}

impl PPU {
    pub fn new(system_type: SystemType) -> PPU {
        match system_type {
            SystemType::DMG => PPU {
                vram: BankedMemory::new_empty(false, 1, 0x2000, true, String::from("dmg vram")),
                oam: vec![0u8; 0xa0],
                screen: vec![0u32; GB_SCREEN_WIDTH * GB_SCREEN_HEIGHT],

                ppu_enabled: false,
                window_tilemap_offset: false,
                window_enable: false,
                bg_window_tiledata_offset: false,
                bg_tilemap_offset: false,
                obj_size_is_8x16: false,
                obj_enable: false,
                bg_window_priority: false,

                scroll_x: W(0),
                scroll_y: W(0),

                current_x: W(0),
                current_y: W(0),
                current_window_y: W(0),

                bg_colors: [COLORS[0], COLORS[1], COLORS[2], COLORS[3]],
                obp1_colors: [COLORS[0], COLORS[1], COLORS[2], COLORS[3]],
                obp2_colors: [COLORS[0], COLORS[1], COLORS[2], COLORS[3]],

                current_mode_cycles: 0,
                mode_3_extra_dots: 0,
                current_mode: PPUMode::OAMScan,

                ly_stat_int: false,
                oam_stat_int: false,
                hblank_stat_int: false,
                vblank_stat_int: false,

                ly_compare: 0,

                dbg_tiledata_bg_swap: false,
                dbg_tilemap_bg_swap: false,
                dbg_tiledata_win_swap: false,
                dbg_tilemap_win_swap: false,
                dbg_win_toggle: false,

                window_x: 0,
                window_y: 0,
            },
        }
    }

    pub fn get_lcdc(&self) -> u8 {
        let mut val: u8 = (self.ppu_enabled as u8) << 7;
        val |= (self.window_tilemap_offset as u8) << 6;
        val |= (self.window_enable as u8) << 5;
        val |= (self.bg_window_tiledata_offset as u8) << 4;
        val |= (self.bg_tilemap_offset as u8) << 3;
        val |= (self.obj_size_is_8x16 as u8) << 2;
        val |= (self.obj_enable as u8) << 1;
        val |= (self.bg_window_priority as u8) << 0;
        val
    }

    pub fn set_lcdc(&mut self, value: u8) {
        self.ppu_enabled = value & 0x80 != 0;
        self.window_tilemap_offset = value & 0x40 != 0;
        self.window_enable = value & 0x20 != 0;
        self.bg_window_tiledata_offset = value & 0x10 != 0;
        self.bg_tilemap_offset = value & 0x08 != 0;
        self.obj_size_is_8x16 = value & 0x04 != 0;
        self.obj_enable = value & 0x02 != 0;
        self.bg_window_priority = value & 0x01 != 0;
    }

    pub fn get_current_y(&self) -> u8 {
        self.current_y.0
    }

    pub fn get_bgpal(&self) -> u8 {
        let mut val = self.bg_colors[0] as u8;
        val |= (self.bg_colors[1] as u8).wrapping_shl(2);
        val |= (self.bg_colors[2] as u8).wrapping_shl(4);
        val | (self.bg_colors[3] as u8).wrapping_shl(6)
    }

    pub fn get_obp1(&self) -> u8 {
        let mut val = self.obp1_colors[0] as u8;
        val |= (self.obp1_colors[1] as u8).wrapping_shl(2);
        val |= (self.obp1_colors[2] as u8).wrapping_shl(4);
        val | (self.obp1_colors[3] as u8).wrapping_shl(6)
    }

    pub fn get_obp2(&self) -> u8 {
        let mut val = self.obp2_colors[0] as u8;
        val |= (self.obp2_colors[1] as u8).wrapping_shl(2);
        val |= (self.obp2_colors[2] as u8).wrapping_shl(4);
        val | (self.obp2_colors[3] as u8).wrapping_shl(6)
    }

    pub fn set_bgpal(&mut self, value: u8) {
        self.bg_colors[0] = COLORS[(value & 0x3) as usize];
        self.bg_colors[1] = COLORS[(value.wrapping_shr(2) & 0x3) as usize];
        self.bg_colors[2] = COLORS[(value.wrapping_shr(4) & 0x3) as usize];
        self.bg_colors[3] = COLORS[(value.wrapping_shr(6) & 0x3) as usize];
    }

    pub fn set_obp1(&mut self, value: u8) {
        self.obp1_colors[0] = COLORS[(value & 0x3) as usize];
        self.obp1_colors[1] = COLORS[(value.wrapping_shr(2) & 0x3) as usize];
        self.obp1_colors[2] = COLORS[(value.wrapping_shr(4) & 0x3) as usize];
        self.obp1_colors[3] = COLORS[(value.wrapping_shr(6) & 0x3) as usize];
    }

    pub fn set_obp2(&mut self, value: u8) {
        self.obp2_colors[0] = COLORS[(value & 0x3) as usize];
        self.obp2_colors[1] = COLORS[(value.wrapping_shr(2) & 0x3) as usize];
        self.obp2_colors[2] = COLORS[(value.wrapping_shr(4) & 0x3) as usize];
        self.obp2_colors[3] = COLORS[(value.wrapping_shr(6) & 0x3) as usize];
    }

    pub fn get_stat(&self) -> u8 {
        let mut val: u8 = (self.ly_stat_int as u8) << 6;
        val |= (self.oam_stat_int as u8) << 5;
        val |= (self.vblank_stat_int as u8) << 4;
        val |= (self.hblank_stat_int as u8) << 3;
        val |= ((self.current_y.0 == self.ly_compare) as u8) << 2;
        val | (self.current_mode as u8)
    }

    pub fn set_stat(&mut self, value: u8) {
        self.ly_stat_int = (value & LY_STAT_INT) != 0;
        self.oam_stat_int = (value & OAM_STAT_INT) != 0;
        self.vblank_stat_int = (value & VBLANK_STAT_INT) != 0;
        self.hblank_stat_int = (value & HBLANK_STAT_INT) != 0;
    }

    pub fn get_ly_compare(&self) -> u8 {
        return self.ly_compare;
    }

    pub fn set_ly_compare(&mut self, value: u8) {
        self.ly_compare = value;
    }

    pub fn read_vram_byte(&self, addr: u16) -> u8 {
        if self.current_mode != PPUMode::PixelPut {
            return self.vram.read_byte(addr);
        }
        return super::UNDEFINED_READ;
    }

    pub fn write_vram_byte(&mut self, addr: u16, value: u8) {
        if self.current_mode != PPUMode::PixelPut {
            self.vram.write_byte(addr, value);
        }
    }

    pub fn read_oam_byte(&self, addr: u16) -> u8 {
        if (self.current_mode != PPUMode::PixelPut) && (self.current_mode != PPUMode::OAMScan) {
            return self.oam[addr as usize];
        }
        return super::UNDEFINED_READ;
    }

    pub fn write_oam_byte(&mut self, addr: u16, value: u8) {
        if (self.current_mode != PPUMode::PixelPut) && (self.current_mode != PPUMode::OAMScan) {
            self.oam[addr as usize] = value;
        }
    }

    fn get_tile_line_pixels(&self, tile_addr: u16) -> [u8; 8] {
        let data_1 = self.vram.read_byte(tile_addr);
        let data_2 = self.vram.read_byte(tile_addr.wrapping_add(1));
        let mut ret_pixels: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];

        for pixel in 0..8 {
            let bitmask = 1 << (7 - pixel);
            let pixel_raw_color_1 = if data_1 & bitmask != 0 { 1 } else { 0 };
            let pixel_raw_color_2 = if data_2 & bitmask != 0 { 2 } else { 0 };
            let pixel_raw_color = pixel_raw_color_1 | pixel_raw_color_2;
            ret_pixels[pixel] = pixel_raw_color;
        }

        return ret_pixels;
    }

    fn render_tiles(&mut self, render_window: bool) {
        let start_x;
        let end_x;
        let x_scroll_tile_offset;
        let current_y;
        let y_tile;
        let y_in_tile;
        let mut line_tilemap_offset;

        let this_line_screen_offset;
        let mut screen_put_offset;

        if render_window {
            start_x = self.window_x.wrapping_sub(7);
            current_y = self.current_window_y;
            this_line_screen_offset = (W(self.current_y.0 as u16) * W(GB_SCREEN_WIDTH as u16)).0;
            screen_put_offset = this_line_screen_offset;

            if self.current_y.0 < self.window_y {
                return;
            }
            self.current_window_y += 1;
        } else {
            start_x = self.scroll_x.0;
            current_y = self.current_y + self.scroll_y;
            this_line_screen_offset = (W(self.current_y.0 as u16) * W(GB_SCREEN_WIDTH as u16)).0;
            screen_put_offset = this_line_screen_offset;
        }

        end_x = start_x as u16 + GB_SCREEN_WIDTH as u16 + 8;
        x_scroll_tile_offset = start_x % 8;
        y_tile = W((current_y >> 3).0 as u16);
        y_in_tile = current_y % W(8);
        line_tilemap_offset = (y_tile * W(32)) + W(0x1800);

        if (render_window && self.window_tilemap_offset) ^ self.dbg_tilemap_win_swap {
            line_tilemap_offset += 0x400;
        } else if (!render_window && self.bg_tilemap_offset) ^ self.dbg_tilemap_bg_swap {
            line_tilemap_offset += 0x400;
        }

        for x_tile in (start_x.wrapping_shr(3) as u16)..(end_x.wrapping_shr(3)) {
            let tilemap_address = line_tilemap_offset + W(x_tile & 0xffu16.wrapping_shr(3)); //this is to mask x_tile to a 8 bit value
            let tilemap_data = W(self.vram.read_byte(tilemap_address.0) as u16);
            let mut tiledata_offset = tilemap_data * W(16) + W(y_in_tile.0 as u16) * W(2);

            if render_window {
                if (!self.bg_window_tiledata_offset) ^ self.dbg_tiledata_win_swap {
                    tiledata_offset += 0x1000;
                    if tiledata_offset.0 >= 0x1800 {
                        tiledata_offset -= 0x1000;
                    }
                }
            } else {
                if (!self.bg_window_tiledata_offset) ^ self.dbg_tiledata_bg_swap {
                    tiledata_offset += 0x1000;
                    if tiledata_offset.0 >= 0x1800 {
                        tiledata_offset -= 0x1000;
                    }
                }
            }
            

            let raw_pixels = self.get_tile_line_pixels(tiledata_offset.0);

            for x_in_tile in 0..8u8 {
                let screen_color = self.bg_colors[raw_pixels[x_in_tile as usize] as usize] as u32;
                if x_scroll_tile_offset as u16 > (screen_put_offset + x_in_tile as u16) { continue; }
                let screen_address =
                    (screen_put_offset + x_in_tile as u16 - x_scroll_tile_offset as u16) as usize;

                if (screen_address < 23040)
                    && (screen_address >= (this_line_screen_offset as usize))
                    && (screen_address < ((this_line_screen_offset as usize + GB_SCREEN_WIDTH) as usize))
                {
                    if self.bg_window_priority {
                        self.screen[screen_address] = screen_color;
                    } else {
                        self.screen[screen_address] = Color::White as u32;
                    }
                }
            }

            screen_put_offset += 8;
        }
    }

    fn render_sprites(&mut self) {
        let mut sprites_drawn_this_line = 0;

        for sprite_idx in (0..self.oam.len()).step_by(4) {
            if sprites_drawn_this_line >= 10 {
                break;
            }

            let sprite_ypos = self.oam[sprite_idx];
            let sprite_xpos = self.oam[sprite_idx + 1];
            let sprite_tileidx = self.oam[sprite_idx + 2];
            let sprite_attribs = self.oam[sprite_idx + 3];

            let sprite_priority = sprite_attribs & 0x80 != 0;
            let y_flip = sprite_attribs & 0x40 != 0;
            let x_flip = sprite_attribs & 0x20 != 0;
            let sprite_pal = sprite_attribs & 0x10 != 0;

            let line_in_sprite = 16u8
                .wrapping_add(self.current_y.0)
                .wrapping_sub(sprite_ypos);

            if (self.obj_size_is_8x16 && line_in_sprite < 16)
                || (!self.obj_size_is_8x16 && line_in_sprite < 8)
            {
                let raw_pixels = if y_flip {
                    self.get_tile_line_pixels((sprite_tileidx as u16).wrapping_shl(4) + if self.obj_size_is_8x16 { 32 } else { 16 } - line_in_sprite as u16 * 2)
                } else {
                    self.get_tile_line_pixels((sprite_tileidx as u16).wrapping_shl(4) + line_in_sprite as u16 * 2)
                };

                let actual_start_x = sprite_xpos.wrapping_sub(8);
                let screen_offset = self.current_y.0 as usize * GB_SCREEN_WIDTH;

                for x_in_sprite in 0..8u8 {
                    if (x_in_sprite.wrapping_add(actual_start_x)) > GB_SCREEN_WIDTH as u8 {
                        continue;
                    }

                    let put_idx = screen_offset + actual_start_x as usize + x_in_sprite as usize;
                    let cur_raw_pix_color = if x_flip {
                        raw_pixels[7 - x_in_sprite as usize]
                    } else {
                        raw_pixels[x_in_sprite as usize]
                    };

                    if cur_raw_pix_color == 0 {
                        continue;
                    }

                    let cur_pix_color = if sprite_pal {
                        self.obp2_colors[cur_raw_pix_color as usize]
                    } else {
                        self.obp1_colors[cur_raw_pix_color as usize]
                    };

                    if put_idx < self.screen.len() {
                        self.screen[put_idx] = cur_pix_color as u32;
                    }
                }

                sprites_drawn_this_line += 1;
                self.mode_3_extra_dots += 6;
            }

            if sprite_xpos == 0 || sprite_xpos >= 168 {
                continue;
            }
        }
    }

    fn render_line(&mut self) {
        self.mode_3_extra_dots = 0;
        self.render_tiles(false);
        if self.window_enable ^ self.dbg_win_toggle {
            self.render_tiles(true);
        }
        if self.obj_enable {
            self.render_sprites();
        }
    }

    pub fn run_cycles(&mut self, cycles: u32, other_state: &mut OtherState) -> Option<Vec<u32>> {
        if self.ppu_enabled {
            self.current_mode_cycles += cycles as u64;
            let dots = W(cycles as u8);

            match self.current_mode {
                PPUMode::HBlank => {
                    self.current_x += dots;

                    if self.current_mode_cycles > (HBLANK_MAX_DOTS - self.mode_3_extra_dots) as u64
                    {
                        self.current_mode_cycles = self
                            .current_mode_cycles
                            .wrapping_sub((HBLANK_MAX_DOTS - self.mode_3_extra_dots) as u64);
                        self.current_y += 1;

                        if self.current_y.0 == self.ly_compare {
                            if self.ly_stat_int {
                                other_state.int_flag |= INT_STAT;
                            }
                        }

                        if self.current_y.0 >= GB_SCREEN_HEIGHT as u8 {
                            self.current_mode = PPUMode::VBlank;
                            self.current_window_y = W(0);
                            other_state.int_flag |= INT_VBLANK;

                            if self.vblank_stat_int {
                                other_state.int_flag |= INT_STAT;
                            }
                        } else {
                            self.current_mode = PPUMode::OAMScan;
                            self.current_x = W(0);
                        }
                    }
                }

                PPUMode::VBlank => {
                    self.current_x += dots;

                    if self.current_mode_cycles > LINE_TOTAL_DOTS as u64 {
                        self.current_mode_cycles = self
                            .current_mode_cycles
                            .wrapping_sub(LINE_TOTAL_DOTS as u64);
                        self.current_y += 1;

                        if self.current_y.0 == self.ly_compare {
                            if self.ly_stat_int {
                                other_state.int_flag |= INT_STAT;
                            }
                        }

                        if self.current_y.0 >= (GB_SCREEN_HEIGHT + VBLANK_LINES) as u8 {
                            self.current_mode = PPUMode::OAMScan;
                            self.current_y = W(0);
                            self.current_x = W(0);

                            if self.current_y.0 == self.ly_compare {
                                if self.ly_stat_int {
                                    other_state.int_flag |= INT_STAT;
                                }
                            }

                            if self.oam_stat_int {
                                other_state.int_flag |= INT_STAT;
                            }

                            return Some(self.screen.clone());
                        }
                    }
                }

                PPUMode::OAMScan => {
                    if self.current_mode_cycles > OAM_SCAN_DOTS as u64 {
                        self.current_mode = PPUMode::PixelPut;
                        self.current_mode_cycles =
                            self.current_mode_cycles.wrapping_sub(OAM_SCAN_DOTS as u64);

                        //set mode_3_extra_dots
                    }
                }

                PPUMode::PixelPut => {
                    self.current_x += dots;

                    if self.current_mode_cycles
                        > (PIXEL_PUT_MIN_DOTS + self.mode_3_extra_dots) as u64
                    {
                        self.current_mode = PPUMode::HBlank;
                        self.current_mode_cycles = self
                            .current_mode_cycles
                            .wrapping_sub((PIXEL_PUT_MIN_DOTS + self.mode_3_extra_dots) as u64);
                        self.render_line();

                        if self.hblank_stat_int {
                            other_state.int_flag |= INT_STAT;
                        }
                    }
                }
            }
        }
        return None;
    }

    pub fn is_enabled(&self) -> bool {
        self.ppu_enabled
    }
}
