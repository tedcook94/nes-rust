pub(super) fn and(a: u8, m: u8) -> u8 {
    a & m
}

pub(super) fn asl(v: u8, _: bool) -> (u8, bool) {
    (v << 1, v & 0x80 != 0)
}

pub(super) fn dec(v: u8) -> u8 {
    v.wrapping_sub(1)
}

pub(super) fn eor(a: u8, m: u8) -> u8 {
    a ^ m
}

pub(super) fn inc(v: u8) -> u8 {
    v.wrapping_add(1)
}

pub(super) fn lda(r: &mut super::Registers, v: u8) {
    r.a = v
}

pub(super) fn ldx(r: &mut super::Registers, v: u8) {
    r.x = v
}

pub(super) fn ldy(r: &mut super::Registers, v: u8) {
    r.y = v
}

pub(super) fn lsr(v: u8, _: bool) -> (u8, bool) {
    (v >> 1, v & 0x01 != 0)
}

pub(super) fn ora(a: u8, m: u8) -> u8 {
    a | m
}

pub(super) fn rol(v: u8, c: bool) -> (u8, bool) {
    ((v << 1) | u8::from(c), v & 0x80 != 0)
}

pub(super) fn ror(v: u8, c: bool) -> (u8, bool) {
    ((v >> 1) | (u8::from(c) << 7), v & 0x01 != 0)
}
