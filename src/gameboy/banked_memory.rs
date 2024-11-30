use log::{debug, warn};

pub struct BankedMemory {
    read_only: bool,
    bank_count: u16,
    bank_size: u16,
    current_bank: u16,
    memory_data: Vec<u8>,
    name: String,
}

impl BankedMemory {
    pub fn new_from_arr(
        read_only: bool,
        bank_count: u16,
        bank_size: u16,
        memory_data: Vec<u8>,
        name: String,
    ) -> BankedMemory {
        BankedMemory {
            read_only,
            bank_count,
            bank_size,
            current_bank: 0,
            memory_data,
            name,
        }
    }

    pub fn new_empty(
        read_only: bool,
        bank_count: u16,
        bank_size: u16,
        fill_random: bool,
        name: String
    ) -> BankedMemory {
        let mut ret = BankedMemory {
            read_only,
            bank_count,
            bank_size,
            current_bank: 0,
            memory_data: vec![0u8; (bank_count * bank_size) as usize],
            name,
        };

        if fill_random {
            ret.memory_data.iter_mut().for_each(|i| *i = rand::random());
        }

        return ret;
    }

    pub fn switch_bank(&mut self, new_bank: u16) {
        if new_bank >= self.bank_count {
            debug!(
                "Tried to switch to bank {} when there are only {}, wrapping over to {}, in: {}",
                new_bank,
                self.bank_count,
                new_bank % self.bank_count,
                self.name
            );
            self.current_bank = new_bank % self.bank_count;
        } else {
            self.current_bank = new_bank;
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        if addr > self.bank_size {
            warn!(
                "possible bug, tried to read beyond banked memory size, (addr: {:#06x}, in: {})",
                addr, self.name
            );
            return crate::gameboy::UNDEFINED_READ;
        } else {
            return self.memory_data[self.current_bank as usize * self.bank_size as usize + addr as usize];
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        if self.read_only {
            return;
        }
        if addr > self.bank_size {
            warn!(
                "possible bug, tried to write beyond banked memory size, (addr: {:#06x}, in: {})",
                addr, self.name
            );
            return;
        } else {
            self.memory_data[self.current_bank as usize * self.bank_size as usize + addr as usize] = value;
        }
    }

    pub fn get_bank_count(&self) -> u16 {
        self.bank_count
    }
}
