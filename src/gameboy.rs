use ppu::Color;
use core::num::Wrapping as W;
use game_carts::GameCart;
use banked_memory::BankedMemory;
use log::{debug, error, info};
use minifb::Key;
use ppu::PPU;
use registers::Registers;

use std::{fs, fs::File, io::BufWriter, io::Write};

use self::render::InputKey;

pub mod banked_memory;
pub mod game_carts;
pub mod io_reg;
pub mod opcodes;
pub mod ppu;
pub mod registers;
pub mod render;

#[derive(Debug)]
pub enum SystemType {
    DMG,
}

const ROM_START: u16 = 0x0000;
const ROM_END: u16 = 0x7fff;
const BOOTROM_SIZE: u16 = 0x100;
const VRAM_START: u16 = 0x8000;
const VRAM_END: u16 = 0x9fff;
const CART_RAM_START: u16 = 0xa000;
const CART_RAM_END: u16 = 0xbfff;
const WRAM_START: u16 = 0xc000;
const WRAM_END: u16 = 0xdfff;
const ECHO_WRAM_START: u16 = 0xe000;
const ECHO_WRAM_END: u16 = 0xfdff;
const OAM_START: u16 = 0xfe00;
const OAM_END: u16 = 0xfe9f;
const IO_REG_START: u16 = 0xff00;
const IO_REG_END: u16 = 0xff7f;
const HRAM_START: u16 = 0xff80;
const HRAM_END: u16 = 0xfffe;
const IE_ADDRESS: u16 = 0xffff;
const UNDEFINED_READ: u8 = 0xff;

const INT_VBLANK: u8 = 0x1;
const INT_STAT: u8 = 0x2;
const INT_TIMER: u8 = 0x4;
const INT_SERIAL: u8 = 0x8;
const INT_GAMEPAD: u8 = 0x10;

pub struct Gameboy {
    reg: Registers,
    ppu: PPU,
    bootrom_data: Vec<u8>,
    rom: Box<dyn GameCart>,
    wram: BankedMemory,
    hram: Vec<u8>,
    pc: W<u16>,
    sp: W<u16>,
    ime: bool,
    cycles_pending: u32,
    cycles_run: u128,
    display_frame_cycles: i32,
    other_state: OtherState,
}

pub struct OtherState {
    bootrom_enabled: bool,
    ime_next_cycle: bool,
    int_enable: u8,
    int_flag: u8,
    halted: bool,
    instrs_run: u128,
    counter_div: W<u16>,
    counter_tima: W<u8>,
    counter_tma: u8,
    counter_tac: u8,
    joypad_io_state: u8,
    oam_dma_running: bool,
    oam_dma_start_addr: u16,
    oam_dma_cur_addr: u8,
    input_keys: Vec<InputKey>,
    force_crash: bool,
}

impl OtherState {
    pub fn new() -> OtherState {
        OtherState {
            bootrom_enabled: true,
            ime_next_cycle: false,
            int_enable: 0,
            int_flag: 0,
            halted: false,
            instrs_run: 0,
            counter_div: W(0),
            counter_tima: W(0),
            counter_tma: 0,
            counter_tac: 0,
            joypad_io_state: 0,
            oam_dma_running: false,
            oam_dma_start_addr: 0,
            oam_dma_cur_addr: 0,
            input_keys: Vec::<InputKey>::with_capacity(8),
            force_crash: false,
        }
    }
}

impl Gameboy {
    pub fn new(
        system_type: SystemType,
        rom_file_path: &str,
        bootrom_file_path: &str
    ) -> Gameboy {
        let rom_data = fs::read(rom_file_path).unwrap();
        let bootrom_data = fs::read(bootrom_file_path).unwrap();

        match system_type {
            SystemType::DMG => {
                let mut dmg_ret = Gameboy {
                    reg: Registers::new(),
                    ppu: PPU::new(system_type),
                    rom: game_carts::get_cart(rom_data),
                    bootrom_data,
                    wram: BankedMemory::new_empty(false, 1, 0x2000, true, String::from("dmg wram")),
                    hram: vec![0u8; 0x80],
                    pc: W(0),
                    sp: W(0),
                    ime: false,
                    cycles_pending: 0,
                    cycles_run: 0,
                    display_frame_cycles: 0,
                    other_state: OtherState::new(),
                };

                dmg_ret
                    .other_state
                    .input_keys
                    .push(InputKey::new(Key::Enter)); //start
                dmg_ret
                    .other_state
                    .input_keys
                    .push(InputKey::new(Key::Space)); //sel
                dmg_ret.other_state.input_keys.push(InputKey::new(Key::S)); //b
                dmg_ret.other_state.input_keys.push(InputKey::new(Key::A)); //a
                dmg_ret
                    .other_state
                    .input_keys
                    .push(InputKey::new(Key::Down)); //down
                dmg_ret.other_state.input_keys.push(InputKey::new(Key::Up)); //up
                dmg_ret
                    .other_state
                    .input_keys
                    .push(InputKey::new(Key::Left)); //left
                dmg_ret
                    .other_state
                    .input_keys
                    .push(InputKey::new(Key::Right)); //right
                return dmg_ret;
            }
        }
    }

    pub fn debug(&self, offset_pc: bool) {
        info!("AF: {:#06x}  BC: {:#06x}\nDE: {:#06x}  HL: {:#06x}\nSP: {:#06x}  PC: {:#06x}\nZ: {}  N: {}  H: {}  C: {}", 
        self.reg.get_af(), self.reg.get_bc(), self.reg.get_de(), self.reg.get_hl(), self.sp, if offset_pc { self.pc - W(1) } else { self.pc },
        self.reg.get_flag_z() as i32, self.reg.get_flag_n() as i32, self.reg.get_flag_h() as i32, self.reg.get_flag_c() as i32);
    }

    #[inline(always)]
    pub fn read_byte_raw(&mut self, addr: W<u16>) -> W<u8> {
        let addr = addr.0;

        match addr {
            ROM_START..=ROM_END => {
                if self.other_state.bootrom_enabled && addr < BOOTROM_SIZE {
                    return W(self.bootrom_data[addr as usize]);
                }
                return W(self.rom.read_byte(addr));
            }
            VRAM_START..=VRAM_END => {
                return W(self.ppu.read_vram_byte(addr - VRAM_START));
            }
            CART_RAM_START..=CART_RAM_END => {
                return W(self.rom.read_byte(addr));
            }
            WRAM_START..=WRAM_END => {
                return W(self.wram.read_byte(addr - WRAM_START));
            }
            ECHO_WRAM_START..=ECHO_WRAM_END => {
                return W(self.wram.read_byte(addr - ECHO_WRAM_START));
            }
            OAM_START..=OAM_END => {
                return W(self.ppu.read_oam_byte(addr - OAM_START));
            }
            IO_REG_START..=IO_REG_END => {
                return W(io_reg::read_byte(self, addr as u8));
            }
            HRAM_START..=HRAM_END => {
                return W(self.hram[(addr - HRAM_START) as usize]);
            }
            IE_ADDRESS => {
                return W(self.other_state.int_enable);
            }
            _ => {
                return W(UNDEFINED_READ);
            }
        }
    }

    #[inline(always)]
    pub fn read_byte(&mut self, addr: W<u16>) -> W<u8> {
        self.cycles_pending += 4;

        if self.other_state.oam_dma_running && addr.0 < IO_REG_START {
            return W(UNDEFINED_READ);
        }

        return self.read_byte_raw(addr);
    }

    #[inline(always)]
    pub fn write_byte_raw(&mut self, addr: W<u16>, value: W<u8>) {
        let value = value.0;
        let addr = addr.0;

        match addr {
            ROM_START..=ROM_END => {
                self.rom.write_byte(addr, value);
            }
            VRAM_START..=VRAM_END => {
                self.ppu.write_vram_byte(addr - VRAM_START, value);
            }
            CART_RAM_START..=CART_RAM_END => {
                self.rom.write_byte(addr, value);
            }
            WRAM_START..=WRAM_END => {
                self.wram.write_byte(addr - WRAM_START, value);
            }
            ECHO_WRAM_START..=ECHO_WRAM_END => {
                self.wram.write_byte(addr - ECHO_WRAM_START, value);
            }
            OAM_START..=OAM_END => {
                self.ppu.write_oam_byte(addr - OAM_START, value);
            }
            IO_REG_START..=IO_REG_END => {
                io_reg::write_byte(self, addr as u8, value);
            }
            HRAM_START..=HRAM_END => {
                self.hram[(addr - HRAM_START) as usize] = value;
            }
            IE_ADDRESS => {
                self.other_state.int_enable = value;
            }
            _ => {
                return;
            }
        }
    }

    #[inline(always)]
    pub fn write_byte(&mut self, addr: W<u16>, value: W<u8>) {
        self.cycles_pending += 4;

        if self.other_state.oam_dma_running && addr.0 < IO_REG_START {
            return;
        }

        self.write_byte_raw(addr, value);
    }

    #[inline(always)]
    pub fn read_byte_inc_pc(&mut self) -> W<u8> {
        let value = self.read_byte(self.pc);
        self.pc += 1;
        value
    }

    #[inline(always)]
    pub fn read_short(&mut self, addr: W<u16>) -> W<u16> {
        let value = self.read_byte(addr).0 as u16;
        W(value | (self.read_byte(addr + W(1)).0 as u16).wrapping_shl(8))
    }

    #[inline(always)]
    pub fn write_short(&mut self, addr: W<u16>, value: W<u16>) {
        self.write_byte(addr, W(value.0 as u8));
        self.write_byte(addr + W(1), W((value >> 8).0 as u8));
    }

    #[inline(always)]
    pub fn read_short_inc_pc(&mut self) -> W<u16> {
        let value = self.read_byte_inc_pc().0 as u16;
        W(value | (self.read_byte_inc_pc().0 as u16).wrapping_shl(8))
    }

    #[inline(always)]
    pub fn push_short(&mut self, value: W<u16>) {
        self.sp -= 2;
        self.write_short(self.sp, value);
    }

    #[inline(always)]
    pub fn pop_short(&mut self) -> W<u16> {
        let value = self.read_short(self.sp);
        self.sp += 2;
        value
    }
}

pub fn run_frame<'a>(
    gb: &'a mut Gameboy,
    input_keys: &Vec<InputKey>,
) -> Result<Vec<u32>, &'a str> {
    handle_input(gb, input_keys);
    gb.display_frame_cycles = 70224;
    loop {
        gb.other_state.instrs_run += 1;
        gb.cycles_pending = 0;

        let mut opcode: W<u8> = W(0);

        if !gb.other_state.halted {
            opcode = gb.read_byte_inc_pc();
        } else {
            gb.cycles_pending += 4;
        }

        if gb.other_state.ime_next_cycle {
            gb.other_state.ime_next_cycle = false;
            gb.ime = true;
        }

        if opcodes::run_opcode(gb, opcode) && !gb.other_state.force_crash {
            gb.cycles_run += gb.cycles_pending as u128;

            process_interrupts(gb);

            if !gb.ppu.is_enabled() {
                gb.display_frame_cycles -= gb.cycles_pending as i32;
                if gb.display_frame_cycles <= 0 {
                    gb.display_frame_cycles = 70224;
                    return Ok(vec![ppu::Color::White as u32]);
                }
            }

            if let Some(frame) = gb.ppu.run_cycles(gb.cycles_pending, &mut gb.other_state) {
                return Ok(frame);
            }

            process_timers(gb);

            if gb.other_state.oam_dma_running {
                process_oam_dma(gb);
            }

            //process serial, etc
        } else {
            error!("Invalid opcode {:#04x}!", opcode);
            gb.debug(true);
            error!("Ran for {} cycles.", gb.cycles_run);
            return Err("crashed");
        }
    }
}

pub fn process_interrupts(gb: &mut Gameboy) {
    let interrupts_to_process = gb.other_state.int_enable & gb.other_state.int_flag;

    if (interrupts_to_process & 0x1f) != 0 {
        gb.other_state.halted = false;

        let interrupt_jump_addr: Option<W<u16>>;
        let interrupt_mask: u8;
        if (interrupts_to_process & INT_VBLANK) != 0 {
            interrupt_jump_addr = Some(W(0x40));
            interrupt_mask = !INT_VBLANK;
        } else if (interrupts_to_process & INT_STAT) != 0 {
            interrupt_jump_addr = Some(W(0x48));
            interrupt_mask = !INT_STAT;
        } else if (interrupts_to_process & INT_TIMER) != 0 {
            interrupt_jump_addr = Some(W(0x50));
            interrupt_mask = !INT_TIMER;
        } else if (interrupts_to_process & INT_SERIAL) != 0 {
            interrupt_jump_addr = Some(W(0x58));
            interrupt_mask = !INT_SERIAL;
        } else if (interrupts_to_process & INT_GAMEPAD) != 0 {
            interrupt_jump_addr = Some(W(0x60));
            interrupt_mask = !INT_GAMEPAD;
        } else {
            interrupt_jump_addr = None;
            interrupt_mask = 0xff;
        }

        if gb.ime && interrupt_jump_addr.is_some() {
            gb.cycles_pending += 8;
            gb.push_short(gb.pc);
            gb.cycles_pending += 4;
            gb.ime = false;
            gb.other_state.int_flag &= interrupt_mask;

            let interrupt_jump_addr = interrupt_jump_addr.unwrap();
            gb.pc = interrupt_jump_addr;
        }
    }
}

pub fn process_timers(gb: &mut Gameboy) {
    gb.other_state.counter_div += gb.cycles_pending as u16;
}

pub fn process_oam_dma(gb: &mut Gameboy) {
    for _ in 0..(gb.cycles_pending / 4) {
        if !gb.other_state.oam_dma_running {
            break;
        }

        let addr_src =
            W(gb.other_state.oam_dma_start_addr + gb.other_state.oam_dma_cur_addr as u16);
        let addr_dest = W(OAM_START + gb.other_state.oam_dma_cur_addr as u16);
        let to_write = gb.read_byte_raw(addr_src);
        gb.write_byte_raw(addr_dest, to_write);

        gb.other_state.oam_dma_cur_addr = gb.other_state.oam_dma_cur_addr.wrapping_add(1);
        gb.other_state.oam_dma_running = gb.other_state.oam_dma_cur_addr < 0xA0;
    }


    //debug!("Copied {} bytes for OAM. (cur idx: 0x{:#02x})", gb.cycles_pending / 4, gb.other_state.oam_dma_cur_addr);
}

pub fn handle_input(gb: &mut Gameboy, input_keys: &Vec<InputKey>) {
    for i in input_keys.iter().enumerate() {
        if i.1.get_state_just_changed() {
            if i.0 == 8 {
                if i.1.get_held() {
                    gb.ppu.dbg_tilemap_bg_swap = !gb.ppu.dbg_tilemap_bg_swap;
                }
                continue;
            } else if i.0 == 9 {
                if i.1.get_held() {
                    gb.ppu.dbg_tiledata_bg_swap = !gb.ppu.dbg_tiledata_bg_swap;
                }
                continue;
            } else if i.0 == 10 {
                if i.1.get_held() {
                    gb.ppu.dbg_tilemap_win_swap = !gb.ppu.dbg_tilemap_win_swap;
                }
                continue;
            } else if i.0 == 11 {
                if i.1.get_held() {
                    gb.ppu.dbg_tiledata_win_swap = !gb.ppu.dbg_tiledata_win_swap;
                }
                continue;
            } else if i.0 == 12 {
                if i.1.get_held() {
                    gb.ppu.dbg_win_toggle = !gb.ppu.dbg_win_toggle;
                }
                continue;
            }

            gb.other_state.int_flag |= INT_GAMEPAD;
            gb.other_state.input_keys[i.0].copy_state_from_other(i.1);
        }
    }
}
