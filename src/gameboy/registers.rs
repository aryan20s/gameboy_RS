use core::num::Wrapping as W;

pub struct Registers {
    pub a: W<u8>,
    f: W<u8>,
    pub b: W<u8>,
    pub c: W<u8>,
    pub d: W<u8>,
    pub e: W<u8>,
    pub h: W<u8>,
    pub l: W<u8>,
}

const FLAG_Z: W<u8> = W(0x80);
const FLAG_N: W<u8> = W(0x40);
const FLAG_H: W<u8> = W(0x20);
const FLAG_C: W<u8> = W(0x10);
const FLAGS_ALL: W<u8> = W(FLAG_C.0 | FLAG_H.0 | FLAG_N.0 | FLAG_Z.0);

impl Registers {
    pub fn new() -> Registers {
        return Registers {
            a: W(0u8),
            f: W(0u8),
            b: W(0u8),
            c: W(0u8),
            d: W(0u8),
            e: W(0u8),
            h: W(0u8),
            l: W(0u8),
        };
    }

    #[inline(always)]
    pub fn get_f(&self) -> W<u8> {
        self.f & FLAGS_ALL
    }

    #[inline(always)]
    pub fn set_f(&mut self, value: W<u8>) {
        self.f = value & FLAGS_ALL;
    }

    #[inline(always)]
    pub fn get_af(&self) -> W<u16> {
        let mut val: u16 = (self.f.0 as u16) & FLAGS_ALL.0 as u16;
        val |= (self.a.0 as u16) << 8;
        W(val)
    }

    #[inline(always)]
    pub fn get_bc(&self) -> W<u16> {
        let mut val: u16 = self.c.0 as u16;
        val |= (self.b.0 as u16) << 8;
        W(val)
    }

    #[inline(always)]
    pub fn get_de(&self) -> W<u16> {
        let mut val: u16 = self.e.0 as u16;
        val |= (self.d.0 as u16) << 8;
        W(val)
    }

    #[inline(always)]
    pub fn get_hl(&self) -> W<u16> {
        let mut val: u16 = self.l.0 as u16;
        val |= (self.h.0 as u16) << 8;
        W(val)
    }

    #[inline(always)]
    pub fn set_af(&mut self, value: W<u16>) {
        let value = value.0;
        self.a = W((value >> 8) as u8);
        self.f = W(value as u8) & FLAGS_ALL;
    }

    #[inline(always)]
    pub fn set_bc(&mut self, value: W<u16>) {
        let value = value.0;
        self.b = W((value >> 8) as u8);
        self.c = W(value as u8);
    }

    #[inline(always)]
    pub fn set_de(&mut self, value: W<u16>) {
        let value = value.0;
        self.d = W((value >> 8) as u8);
        self.e = W(value as u8);
    }

    #[inline(always)]
    pub fn set_hl(&mut self, value: W<u16>) {
        let value = value.0;
        self.h = W((value >> 8) as u8);
        self.l = W(value as u8);
    }

    #[inline(always)]
    pub fn unset_all_flags(&mut self) {
        self.f = W(0u8);
    }

    #[inline(always)]
    pub fn get_flag_z(&self) -> bool {
        (self.f & FLAG_Z).0 != 0
    }

    #[inline(always)]
    pub fn get_flag_n(&self) -> bool {
        (self.f & FLAG_N).0 != 0
    }

    #[inline(always)]
    pub fn get_flag_h(&self) -> bool {
        (self.f & FLAG_H).0 != 0
    }

    #[inline(always)]
    pub fn get_flag_c(&self) -> bool {
        (self.f & FLAG_C).0 != 0
    }

    #[inline(always)]
    pub fn set_flag_z(&mut self) {
        self.f |= FLAG_Z;
    }

    #[inline(always)]
    pub fn set_flag_n(&mut self) {
        self.f |= FLAG_N;
    }

    #[inline(always)]
    pub fn set_flag_h(&mut self) {
        self.f |= FLAG_H;
    }

    #[inline(always)]
    pub fn set_flag_c(&mut self) {
        self.f |= FLAG_C;
    }

    #[inline(always)]
    pub fn unset_flag_z(&mut self) {
        self.f &= !FLAG_Z;
    }

    #[inline(always)]
    pub fn unset_flag_n(&mut self) {
        self.f &= !FLAG_N;
    }

    #[inline(always)]
    pub fn unset_flag_h(&mut self) {
        self.f &= !FLAG_H;
    }

    #[inline(always)]
    pub fn unset_flag_c(&mut self) {
        self.f &= !FLAG_C;
    }
}
