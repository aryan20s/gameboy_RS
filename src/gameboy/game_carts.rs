use crate::gameboy::UNDEFINED_READ;
use log::{debug, trace};

const CART_RAM_START: usize = 0xa000;

pub fn get_cart(rom_data: Vec<u8>) -> Box<dyn GameCart> {
    let header_mapper_byte = rom_data[0x147];

    match header_mapper_byte {
        0x00 => Box::new(NoMapperCart::new(rom_data)),
        0x01..=0x03 => Box::new(MBC1Cart::new(rom_data)),
        0x0F..=0x13 => Box::new(MBC3Cart::new(rom_data)),
        _ => Box::new(MBC1Cart::new(rom_data))
    }
}

pub trait GameCart {
    fn read_byte(&self, addr: u16) -> u8;
    fn write_byte(&mut self, addr: u16, val: u8);
}

struct NoMapperCart {
    rom_data: Vec<u8>
}

impl NoMapperCart {
    pub fn new(rom_data: Vec<u8>) -> NoMapperCart {
        Self { rom_data }
    }
}

impl GameCart for NoMapperCart {
    fn read_byte(&self, addr: u16) -> u8 {
        if addr < 0x8000 { self.rom_data[addr as usize] } else { UNDEFINED_READ }
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        // self.rom_data[addr as usize] = val;
    }
}

struct MBC1Cart {
    ram_enabled: bool,
    banking_mode_adv: bool,
    rom_bank_number_5b: u8,
    rom_ram_bank_number_2b: u8,
    rom_data: Vec<u8>,
    ram_data: Vec<u8>
}

impl MBC1Cart {
    pub fn new(rom_data: Vec<u8>) -> MBC1Cart {
        let ram_size = match rom_data[0x149] {
            0x00 => 0,
            0x02 => 0x2000,
            0x03 => 0x8000,
            _ => 0
        };

        Self {
            ram_enabled: false,
            banking_mode_adv: false,
            rom_bank_number_5b: 0,
            rom_ram_bank_number_2b: 0,
            rom_data,
            ram_data: vec![0u8; ram_size]
        }
    }

    fn resolve_addr(&self, addr: u16) -> usize {
        if addr < 0x8000 {
            let ret_val = if addr < 0x4000 {
                if self.banking_mode_adv {
                    (self.rom_ram_bank_number_2b as usize * 0x80000) + addr as usize 
                } else {
                    addr as usize
                }
            } else {
                (self.rom_ram_bank_number_2b as usize * 0x80000) + (self.rom_bank_number_5b as usize * 0x4000) + addr as usize - 0x4000
            };
            
            return ret_val;
        } else {
            return if self.banking_mode_adv {
                (self.rom_ram_bank_number_2b as usize * 0x4000) + addr as usize - CART_RAM_START
            } else {
                addr as usize
            };
        }
    }
}

impl GameCart for MBC1Cart {
    fn read_byte(&self, addr: u16) -> u8 {
        let mapped_addr = self.resolve_addr(addr);
        
        match addr {
            0x0000..=0x7FFF => {
                self.rom_data[mapped_addr % self.rom_data.len()]
            }
            0xA000..=0xBFFF => {
                let ram_size = self.ram_data.len();
                if ram_size == 0 || !self.ram_enabled {
                    UNDEFINED_READ
                } else {
                    self.ram_data[mapped_addr % self.ram_data.len()]
                }
            }
            _ => {
                panic!("Unimplemented MBC1 read addr 0x{:#04x}!", addr);
            }
        }
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => {
                if (val & 0xA) != 0 {
                    self.ram_enabled = true;
                } else {
                    self.ram_enabled = false;
                }
            }
            0x2000..=0x3FFF => {
                self.rom_bank_number_5b = val & 0x1F;
                if self.rom_bank_number_5b == 0 {
                    self.rom_bank_number_5b = 1;
                }
            }
            0x4000..=0x5FFF => {
                self.rom_ram_bank_number_2b = val & 0x03;
            }
            0x6000..=0x7FFF => {
                self.banking_mode_adv = (val & 0x01) != 0;
            }
            0xA000..=0xBFFF => {
                let ram_addr = self.resolve_addr(addr);
                let ram_size = self.ram_data.len();

                if ram_size != 0 && self.ram_enabled {
                    self.ram_data[ram_addr % ram_size] = val;
                }
            }
            _ => {
                panic!("Unimplemented MBC1 write 0x{:02x} at 0x{:#04x}!", val, addr);
            }
        }   
    }
}

struct MBC3Cart {
    ram_timer_enabled: bool,
    rom_bank_number_7b: u8,
    ram_bank_number_2b: u8,
    rom_data: Vec<u8>,
    ram_data: Vec<u8>
}

impl MBC3Cart {
    pub fn new(rom_data: Vec<u8>) -> MBC3Cart {
        let ram_size = match rom_data[0x149] {
            0x00 => 0,
            0x02 => 0x2000,
            0x03 => 0x8000,
            _ => 0
        };

        Self {
            ram_timer_enabled: false,
            rom_bank_number_7b: 0,
            ram_bank_number_2b: 0,
            rom_data,
            ram_data: vec![0u8; ram_size]
        }
    }

    fn resolve_addr(&self, addr: u16) -> usize {
        match addr {
            0x0000..=0x3FFF => {
                addr as usize
            },
            0x4000..=0x7FFF => {
                (self.rom_bank_number_7b as usize * 0x4000) + addr as usize - 0x4000
            },
            _ => {
                (self.ram_bank_number_2b as usize * 0x4000) + addr as usize - CART_RAM_START
            }
        }
    }
}

impl GameCart for MBC3Cart {
    fn read_byte(&self, addr: u16) -> u8 {
        let mapped_addr = self.resolve_addr(addr);
        
        match addr {
            0x0000..=0x7FFF => {
                self.rom_data[mapped_addr % self.rom_data.len()]
            }
            0xA000..=0xBFFF => {
                let ram_size = self.ram_data.len();
                if ram_size == 0 {
                    UNDEFINED_READ
                } else {
                    self.ram_data[mapped_addr % self.ram_data.len()]
                }
            }
            _ => {
                panic!("Unimplemented MBC3 read addr 0x{:#04x}!", addr);
            }
        }
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => {
                if (val & 0xA) != 0 {
                    self.ram_timer_enabled = true;
                } else {
                    self.ram_timer_enabled = false;
                }
            }
            0x2000..=0x3FFF => {
                self.rom_bank_number_7b = val & 0x7F;
                if self.rom_bank_number_7b == 0 {
                    self.rom_bank_number_7b = 1;
                }
            }
            0x4000..=0x5FFF => {
                self.ram_bank_number_2b = val & 0x03;
            }
            0x6000..=0x7FFF => {
                // TODO timers
            }
            0xA000..=0xBFFF => {
                let ram_addr = self.resolve_addr(addr);
                let ram_size = self.ram_data.len();

                if ram_size != 0 && self.ram_timer_enabled {
                    self.ram_data[ram_addr % ram_size] = val;
                }
            }
            _ => {
                panic!("Unimplemented MBC3 write 0x{:02x} at 0x{:#04x}!", val, addr);
            }
        }   
    }
}