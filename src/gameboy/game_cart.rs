use super::banked_memory::BankedMemory;
use crate::gameboy::UNDEFINED_READ;

use log::debug;

#[derive(Debug, Clone, Copy)]
pub enum MapperType {
    NoMapper,
    MBC1,
    MBC3
}

pub struct MBC1Registers {
    ram_enabled: bool,
    cur_rom_bank_5b: u8,
    ram_rom_bank_2b: u8,
    banking_mode: bool
}

impl MBC1Registers {
    pub fn new() -> Self {
        Self { ram_enabled: false, cur_rom_bank_5b: 0, ram_rom_bank_2b: 0, banking_mode: false }
    }
}

const ROM_BANK_0_START: u16 = 0x0000;
const ROM_BANK_0_END: u16 = 0x3fff;
const ROM_BANK_OTHER_START: u16 = 0x4000;
const ROM_BANK_OTHER_END: u16 = 0x7fff;
const CART_RAM_END: u16 = 0x1fff;

pub struct GameCart {
    rom_data_main_bank: Vec<u8>,
    rom_data_all_banks: BankedMemory,
    cart_ram: Option<BankedMemory>,
    mapper_type: MapperType,

    mbc_1_reg: Option<MBC1Registers>
}

impl GameCart {
    pub fn new(mapper_type: MapperType, rom_data: Vec<u8>, cart_ram_bank_count: u8) -> GameCart {
        let mut ret_cart = match mapper_type {
            MapperType::NoMapper => {
                if rom_data.len() < 0x8000 {
                    panic!("rom size is less than expected for {:?}!", mapper_type);
                }
                let mut main_rom_bank_data = vec![0u8; 0x4000];
                main_rom_bank_data.copy_from_slice(&rom_data[0..0x4000]);
                let mut other_rom_banks_data = vec![0u8; 0x4000];
                other_rom_banks_data.copy_from_slice(&rom_data[0x4000..0x8000]);

                GameCart {
                    rom_data_main_bank: main_rom_bank_data,
                    rom_data_all_banks: BankedMemory::new_from_arr(
                        true,
                        1,
                        0x4000,
                        other_rom_banks_data,
                        String::from("rom_data_0x4000-0x8000"),
                    ),
                    cart_ram: None,
                    mapper_type,
                    mbc_1_reg: None
                }
            }

            MapperType::MBC1 => {
                if rom_data.len() < 0x8000 {
                    panic!("rom size is less than expected for {:?}!", mapper_type);
                }
                let mut main_rom_bank_data = vec![0u8; 0x4000];
                main_rom_bank_data.copy_from_slice(&rom_data[0..0x4000]);
                let rom_data_len = rom_data.len();
                let mut other_rom_banks_data = vec![0u8; rom_data_len];
                other_rom_banks_data.copy_from_slice(&rom_data[0x0000..rom_data_len]);

                let mut rom_data_all_banks = BankedMemory::new_from_arr(
                    true,
                    (rom_data_len >> 14) as u16,
                    0x4000,
                    other_rom_banks_data,
                    String::from("rom_data_all_banks")
                );
                rom_data_all_banks.switch_bank(1);

                GameCart {
                    rom_data_main_bank: main_rom_bank_data,
                    rom_data_all_banks,
                    cart_ram: None,
                    mapper_type,
                    mbc_1_reg: Some(MBC1Registers::new())
                }
            }

            MapperType::MBC3 => {
                if rom_data.len() < 0x8000 {
                    panic!("rom size is less than expected for {:?}!", mapper_type);
                }
                let mut main_rom_bank_data = vec![0u8; 0x4000];
                main_rom_bank_data.copy_from_slice(&rom_data[0..0x4000]);
                let rom_data_len = rom_data.len();
                let mut other_rom_banks_data = vec![0u8; rom_data_len];
                other_rom_banks_data.copy_from_slice(&rom_data[0x0000..rom_data_len]);

                let mut rom_data_all_banks = BankedMemory::new_from_arr(
                    true,
                    (rom_data_len >> 14) as u16,
                    0x4000,
                    other_rom_banks_data,
                    String::from("rom_data_all_banks")
                );
                rom_data_all_banks.switch_bank(1);

                GameCart {
                    rom_data_main_bank: main_rom_bank_data,
                    rom_data_all_banks,
                    cart_ram: None,
                    mapper_type,
                    mbc_1_reg: None
                }
            }
        };

        if cart_ram_bank_count != 0 {
            ret_cart.cart_ram = Some(BankedMemory::new_empty(false, cart_ram_bank_count as u16, 0x2000, String::from("cart_ram")));
        }

        ret_cart
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            ROM_BANK_0_START..=ROM_BANK_0_END => {
                return self.rom_data_main_bank[addr as usize];
            }
            ROM_BANK_OTHER_START..=ROM_BANK_OTHER_END => {
                return self
                    .rom_data_all_banks
                    .read_byte(addr - ROM_BANK_OTHER_START);
            }
            _ => {
                panic!("tried to read from addr {:#06x} of cartridge rom!", addr);
            }
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match self.mapper_type {
            MapperType::NoMapper => {
                match addr {
                    ROM_BANK_0_START..=ROM_BANK_OTHER_END => {
                    }
                    _ => {
                        panic!("tried to write to addr {:#06x} of cartridge rom!", addr);
                    }
                }
            }

            MapperType::MBC1 => {
                match addr {
                    0x0000..=0x1fff => {
                        self.mbc_1_reg.as_mut().unwrap().ram_enabled = (value & 0xa) == 0xa;
                    }
                    0x2000..=0x3fff => {
                        let mut final_bank = (value & 0x1f) as u16;  
                        if final_bank == 0 {
                            final_bank = 1;
                        }

                        final_bank %= self.rom_data_all_banks.get_bank_count();
                        self.rom_data_all_banks.switch_bank(final_bank);
                        self.mbc_1_reg.as_mut().unwrap().cur_rom_bank_5b = (final_bank & 0x1f) as u8;
                    }
                    0x4000..=0x5fff => {
                        self.mbc_1_reg.as_mut().unwrap().ram_rom_bank_2b = value & 0x3;
                        //self.rom_data_all_banks.switch_bank((self.mbc_1_reg.as_ref().unwrap().ram_rom_bank_2b.wrapping_shl(5) | self.mbc_1_reg.as_ref().unwrap().cur_rom_bank_5b) as u16);
                    }
                    0x6000..=0x7fff => {
                        self.mbc_1_reg.as_mut().unwrap().banking_mode = (value & 0x1) == 0x1;
                    }
                    _ => {
                        debug!(
                            "unimpl rom write {:#04x} to {:#06x}, mapper: {:?}",
                            value, addr, self.mapper_type
                        );
                    }
                }
            }

            MapperType::MBC3 => {
                match addr {
                    0x2000..=0x3fff => {
                        let mut final_bank = (value & 0x1f) as u16;  
                        if final_bank == 0 {
                            final_bank = 1;
                        }

                        final_bank %= self.rom_data_all_banks.get_bank_count();
                        self.rom_data_all_banks.switch_bank(final_bank);
                    }
                    _ => {
                        debug!(
                            "unimpl rom write {:#04x} to {:#06x}, mapper: {:?}",
                            value, addr, self.mapper_type
                        );
                    }
                }
            }
        }
    }

    pub fn read_cart_ram_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=CART_RAM_END => match &self.cart_ram {
                None => UNDEFINED_READ,

                Some(cart_ram) => cart_ram.read_byte(addr),
            },
            _ => {
                panic!("tried to read from addr {:#06x} of cartridge ram!", addr);
            }
        }
    }

    pub fn write_cart_ram_byte(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=CART_RAM_END => match &mut self.cart_ram {
                None => {}

                Some(cart_ram) => {
                    cart_ram.write_byte(addr, value);
                }
            },
            _ => {
                panic!("tried to write to addr {:#06x} of cartridge ram!", addr);
            }
        }
    }

    pub fn get_mapper(&self) -> MapperType {
        self.mapper_type
    }
}
