use core::num::Wrapping as W;

use super::{banked_memory::BankedMemory, SystemType, OtherState, INT_VBLANK, INT_STAT};

use log::debug;

#[derive(Clone, Copy, Debug)]
enum Color {
    Black = 0x00000000,
    DGray = 0x00555555,
    LGray = 0x00aaaaaa,
    White = 0x00ffffff
}

#[derive(PartialEq, Clone, Copy)]
enum PPUMode {
    HBlank = 0,
    VBlank = 1,
    OAMScan = 2,
    PixelPut = 3
}

const COLORS: [Color; 4] =  [Color::White, Color::LGray, Color::DGray, Color::Black];
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

    ppu_enabled: bool,            //lcdc
    window_tilemap_offset: bool,
    window_enable: bool,
    bg_window_tiledata_offset: bool,
    bg_tilemap_offset: bool,

    pub scroll_x: W<u8>, //scx
    pub scroll_y: W<u8>, //scy
    
    current_x: W<u8>,
    current_y: W<u8>, //ly

    bg_colors: [Color; 4], //bgpal

    current_mode_cycles: u64,
    mode_3_extra_dots: u16,
    current_mode: PPUMode,

    ly_stat_int: bool,    //lcd stat
    oam_stat_int: bool,
    hblank_stat_int: bool,
    vblank_stat_int: bool,

    ly_compare: u8, //lyc

    pub window_x: u8, //wx
    pub window_y: u8 //wy
}

impl PPU {
    pub fn new(system_type: SystemType) -> PPU {
        match system_type {
            SystemType::DMG => PPU {
                vram: BankedMemory::new_empty(false, 1, 0x2000, String::from("dmg vram")),
                oam: vec![0u8; 0xa0],
                screen: vec![0u32; GB_SCREEN_WIDTH * GB_SCREEN_HEIGHT],

                ppu_enabled: false,
                window_tilemap_offset: false,
                window_enable: false,
                bg_window_tiledata_offset: false,
                bg_tilemap_offset: false,
                
                scroll_x: W(0),
                scroll_y: W(0),

                current_x: W(0),
                current_y: W(0),

                bg_colors: [COLORS[0], COLORS[1], COLORS[2], COLORS[3]],

                current_mode_cycles: 0,
                mode_3_extra_dots: 0,
                current_mode: PPUMode::OAMScan,

                ly_stat_int: false,
                oam_stat_int: false,
                hblank_stat_int: false,
                vblank_stat_int: false,
                
                ly_compare: 0,

                window_x: 0,
                window_y: 0
            },
        }
    }

    pub fn get_lcdc(&self) -> u8 {
        let mut val: u8 = (self.ppu_enabled as u8) << 7;
        val |= (self.window_tilemap_offset as u8) << 6;
        val |= (self.window_enable as u8) << 6;
        val |= (!self.bg_window_tiledata_offset as u8) << 4;
        val |= (self.bg_tilemap_offset as u8) << 3;
        val
    }

    pub fn set_lcdc(&mut self, value: u8) {
        self.ppu_enabled = value & 0x80 != 0;
        self.window_tilemap_offset = value & 0x40 != 0;
        self.window_enable = value & 0x20 != 0;
        self.bg_window_tiledata_offset = value & 0x10 == 0;
        self.bg_tilemap_offset = value & 0x08 != 0;
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

    pub fn set_bgpal(&mut self, value: u8) {
        self.bg_colors[0] = COLORS[(value & 0x3) as usize];
        self.bg_colors[1] = COLORS[(value.wrapping_shr(2) & 0x3) as usize];
        self.bg_colors[2] = COLORS[(value.wrapping_shr(4) & 0x3) as usize];
        self.bg_colors[3] = COLORS[(value.wrapping_shr(6) & 0x3) as usize];
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

    fn render_bg(&mut self) {
        let start_x = self.scroll_x;
        let end_x = start_x.0 as u16 + 160;
        let start_x = start_x.0;
        let x_scroll_tile_offset = start_x % 8; 

        let current_y = self.current_y + self.scroll_y;
        let y_tile = W((current_y >> 3).0 as u16);
        let y_in_tile = current_y % W(8);
        let mut line_tilemap_offset = (y_tile * W(32)) + W(0x1800);
        if self.bg_tilemap_offset { line_tilemap_offset += 0x400; }

        let this_line_screen_offset = (W(self.current_y.0 as u16) * W(GB_SCREEN_WIDTH as u16)).0;
        let mut screen_put_offset = this_line_screen_offset;
        
        for x_tile in (start_x.wrapping_shr(3) as u16)..(end_x.wrapping_shr(3)) {
            let tilemap_address = line_tilemap_offset + W(x_tile & 0xffu16.wrapping_shr(3)); //this is to mask x_tile to a 8 bit value
            let tilemap_data = W(self.vram.read_byte(tilemap_address.0) as u16);
            let mut tiledata_offset = tilemap_data * W(16) + W(y_in_tile.0 as u16) * W(2);
            if self.bg_window_tiledata_offset { 
                tiledata_offset += 0x1000;
                if tiledata_offset.0 >= 0x1800 {
                    tiledata_offset -= 0x1000;
                }
            }

            let data_1 = self.vram.read_byte(tiledata_offset.0); tiledata_offset += 1;
            let data_2 = self.vram.read_byte(tiledata_offset.0);

            for x_in_tile in x_scroll_tile_offset..8u8 {
                let bitmask = 1 << (7 - x_in_tile);
                let pixel_raw_color_1 = if data_1 & bitmask != 0 { 1 } else { 0 };
                let pixel_raw_color_2 = if data_2 & bitmask != 0 { 2 } else { 0 };
                let pixel_raw_color = pixel_raw_color_1 | pixel_raw_color_2;
                let screen_color = self.bg_colors[pixel_raw_color] as u32;
                
                let screen_address = (screen_put_offset + x_in_tile as u16 - x_scroll_tile_offset as u16) as usize;

                if screen_address < 23040 {
                    self.screen[screen_address] = screen_color;
                }
            }

            for x_in_tile in 0..x_scroll_tile_offset {
                let bitmask = 1 << (7 - x_in_tile);
                let pixel_raw_color_1 = if data_1 & bitmask != 0 { 1 } else { 0 };
                let pixel_raw_color_2 = if data_2 & bitmask != 0 { 2 } else { 0 };
                let pixel_raw_color = pixel_raw_color_1 | pixel_raw_color_2;
                let screen_color = self.bg_colors[pixel_raw_color] as u32;

                if (screen_put_offset + x_in_tile as u16) >= x_scroll_tile_offset as u16 {
                    let screen_address = ((screen_put_offset + x_in_tile as u16) - x_scroll_tile_offset as u16) as usize;

                    if (screen_address < 23040) && (screen_address >= (this_line_screen_offset as usize)) {
                        self.screen[screen_address] = screen_color;
                    }
                }
            }

            screen_put_offset += 8;
        }
    }

    fn render_window(&mut self) {
        let start_x = self.window_x.wrapping_sub(7);
        let end_x = start_x as u16 + 160;
        let start_x = start_x;
        let x_scroll_tile_offset = start_x % 8; 

        let current_y = self.current_y + W(self.window_y);
        let y_tile = W((current_y >> 3).0 as u16);
        let y_in_tile = current_y % W(8);
        let mut line_tilemap_offset = (y_tile * W(32)) + W(0x1800);
        if self.window_tilemap_offset { line_tilemap_offset += 0x400; }

        let this_line_screen_offset = (W(self.current_y.0 as u16) * W(GB_SCREEN_WIDTH as u16)).0;
        let mut screen_put_offset = this_line_screen_offset;
        
        for x_tile in (start_x.wrapping_shr(3) as u16)..(end_x.wrapping_shr(3)) {
            let tilemap_address = line_tilemap_offset + W(x_tile & 0xffu16.wrapping_shr(3)); //this is to mask x_tile to a 8 bit value
            let tilemap_data = W(self.vram.read_byte(tilemap_address.0) as u16);
            let mut tiledata_offset = tilemap_data * W(16) + W(y_in_tile.0 as u16) * W(2);
            if self.bg_window_tiledata_offset { 
                tiledata_offset += 0x1000;
                if tiledata_offset.0 >= 0x1800 {
                    tiledata_offset -= 0x1000;
                }
            }

            let data_1 = self.vram.read_byte(tiledata_offset.0); tiledata_offset += 1;
            let data_2 = self.vram.read_byte(tiledata_offset.0);

            for x_in_tile in x_scroll_tile_offset..8u8 {
                let bitmask = 1 << (7 - x_in_tile);
                let pixel_raw_color_1 = if data_1 & bitmask != 0 { 1 } else { 0 };
                let pixel_raw_color_2 = if data_2 & bitmask != 0 { 2 } else { 0 };
                let pixel_raw_color = pixel_raw_color_1 | pixel_raw_color_2;
                let screen_color = self.bg_colors[pixel_raw_color] as u32;
                
                let screen_address = (screen_put_offset + x_in_tile as u16 - x_scroll_tile_offset as u16) as usize;

                if screen_address < 23040 {
                    self.screen[screen_address] = screen_color;
                }
            }

            for x_in_tile in 0..x_scroll_tile_offset {
                let bitmask = 1 << (7 - x_in_tile);
                let pixel_raw_color_1 = if data_1 & bitmask != 0 { 1 } else { 0 };
                let pixel_raw_color_2 = if data_2 & bitmask != 0 { 2 } else { 0 };
                let pixel_raw_color = pixel_raw_color_1 | pixel_raw_color_2;
                let screen_color = self.bg_colors[pixel_raw_color] as u32;

                if (screen_put_offset + x_in_tile as u16) >= x_scroll_tile_offset as u16 {
                    let screen_address = ((screen_put_offset + x_in_tile as u16) - x_scroll_tile_offset as u16) as usize;

                    if (screen_address < 23040) && (screen_address >= (this_line_screen_offset as usize)) {
                        self.screen[screen_address] = screen_color;
                    }
                }
            }

            screen_put_offset += 8;
        }
    }

    fn render_line(&mut self) {
        self.render_bg();
        if self.window_enable {
            self.render_window();
        }
    }

    pub fn run_cycles(&mut self, cycles: u32, other_state: &mut OtherState) -> Option<Vec<u32>> {
        if self.ppu_enabled {
            self.current_mode_cycles += cycles as u64;
            let dots = W(cycles as u8);

            match self.current_mode {
                PPUMode::HBlank => {
                    self.current_x += dots;

                    if self.current_mode_cycles > (HBLANK_MAX_DOTS - self.mode_3_extra_dots) as u64 {
                        self.current_mode_cycles = self.current_mode_cycles.wrapping_sub((HBLANK_MAX_DOTS - self.mode_3_extra_dots) as u64);
                        self.current_y += 1;

                        if self.current_y.0 == self.ly_compare {
                            if self.ly_stat_int {
                                other_state.int_flag |= INT_STAT;
                            }
                        }

                        if self.current_y.0 >= GB_SCREEN_HEIGHT as u8 {
                            self.current_mode = PPUMode::VBlank;
                            other_state.int_flag |= INT_VBLANK;
                            
                            if self.vblank_stat_int  {
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
                        self.current_mode_cycles = self.current_mode_cycles.wrapping_sub(LINE_TOTAL_DOTS as u64);
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

                            if self.oam_stat_int  {
                                other_state.int_flag |= INT_STAT;
                            }

                            return Some(self.screen.clone());
                        }
                    }
                }

                PPUMode::OAMScan => {
                    if self.current_mode_cycles > OAM_SCAN_DOTS as u64 {
                        self.current_mode = PPUMode::PixelPut;
                        self.current_mode_cycles = self.current_mode_cycles.wrapping_sub(OAM_SCAN_DOTS as u64);

                        //set mode_3_extra_dots
                    }
                }

                PPUMode::PixelPut => {
                    self.current_x += dots;

                    if self.current_mode_cycles > (PIXEL_PUT_MIN_DOTS + self.mode_3_extra_dots) as u64 {
                        self.current_mode = PPUMode::HBlank;
                        self.current_mode_cycles = self.current_mode_cycles.wrapping_sub((PIXEL_PUT_MIN_DOTS + self.mode_3_extra_dots) as u64);
                        self.render_line();

                        if self.hblank_stat_int  {
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
