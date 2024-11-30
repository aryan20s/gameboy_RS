use crate::gameboy::Gameboy;

use log::debug;

#[inline(always)]
pub fn di(gb: &mut Gameboy) {
    gb.ime = false;
}

#[inline(always)]
pub fn ei(gb: &mut Gameboy) {
    gb.other_state.ime_next_cycle = true;
}

#[inline(always)]
pub fn halt(gb: &mut Gameboy) {
    gb.other_state.halted = true;
}

#[inline(always)]
pub fn scf(gb: &mut Gameboy) {
    gb.reg.unset_flag_n();
    gb.reg.unset_flag_h();
    gb.reg.set_flag_c();
}

#[inline(always)]
pub fn ccf(gb: &mut Gameboy) {
    gb.reg.unset_flag_n();
    gb.reg.unset_flag_h();
    if gb.reg.get_flag_c() {
        gb.reg.unset_flag_c();
    } else {
        gb.reg.set_flag_c();
    }
}
