use crate::bus::Bus;

pub struct Cpu {
    registers: Registers,
    status: Status,
    cycle_count: u64,
}

impl Cpu {
    fn new() -> Self {
        Cpu {
            registers: Registers {
                a: 0,
                x: 0,
                y: 0,
                sp: 0xFD,
                pc: 0,
            },
            status: Status(Status::INTERRUPT_DISABLE | Status::UNUSED),
            cycle_count: 0,
        }
    }

    fn step<T: Bus>(&mut self, bus: &mut T) -> u8 {
        0
    }
}

struct Registers {
    a: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,
}

struct Status(u8);

impl Status {
    const CARRY: u8 = 0b0000_0001;
    const ZERO: u8 = 0b0000_0010;
    const INTERRUPT_DISABLE: u8 = 0b0000_0100;
    const DECIMAL_MODE: u8 = 0b0000_1000;
    const BREAK_COMMAND: u8 = 0b0001_0000;
    const OVERFLOW: u8 = 0b0100_0000;
    const NEGATIVE: u8 = 0b1000_0000;
    const UNUSED: u8 = 0b0010_0000;

    fn set(&mut self, flag: u8, val: bool) {
        if val { self.0 |= flag } else { self.0 &= !flag }
    }

    fn is_set(&self, flag: u8) -> bool {
        self.0 & flag != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_powers_on_in_proper_state() {
        let cpu = Cpu::new();
        assert_eq!(cpu.registers.a, 0);
        assert_eq!(cpu.registers.x, 0);
        assert_eq!(cpu.registers.y, 0);
        assert_eq!(cpu.registers.sp, 0xFD);
        assert_eq!(cpu.registers.pc, 0);
        assert_eq!(cpu.status.0, 0x24);
        assert_eq!(cpu.cycle_count, 0);
    }

    #[test]
    fn set_carry_returns_is_set() {
        let mut status = Status(0);
        status.set(Status::CARRY, true);
        assert!(status.is_set(Status::CARRY));
        assert_eq!(status.0, Status::CARRY);
    }

    #[test]
    fn set_zero_returns_is_set() {
        let mut status = Status(0);
        status.set(Status::ZERO, true);
        assert!(status.is_set(Status::ZERO));
        assert_eq!(status.0, Status::ZERO);
    }

    #[test]
    fn set_interrupt_disable_returns_is_set() {
        let mut status = Status(0);
        status.set(Status::INTERRUPT_DISABLE, true);
        assert!(status.is_set(Status::INTERRUPT_DISABLE));
        assert_eq!(status.0, Status::INTERRUPT_DISABLE);
    }

    #[test]
    fn set_decimal_mode_returns_is_set() {
        let mut status = Status(0);
        status.set(Status::DECIMAL_MODE, true);
        assert!(status.is_set(Status::DECIMAL_MODE));
        assert_eq!(status.0, Status::DECIMAL_MODE);
    }

    #[test]
    fn set_break_command_returns_is_set() {
        let mut status = Status(0);
        status.set(Status::BREAK_COMMAND, true);
        assert!(status.is_set(Status::BREAK_COMMAND));
        assert_eq!(status.0, Status::BREAK_COMMAND);
    }

    #[test]
    fn set_overflow_returns_is_set() {
        let mut status = Status(0);
        status.set(Status::OVERFLOW, true);
        assert!(status.is_set(Status::OVERFLOW));
        assert_eq!(status.0, Status::OVERFLOW);
    }

    #[test]
    fn set_negative_returns_is_set() {
        let mut status = Status(0);
        status.set(Status::NEGATIVE, true);
        assert!(status.is_set(Status::NEGATIVE));
        assert_eq!(status.0, Status::NEGATIVE);
    }

    #[test]
    fn set_can_clear_flag() {
        let mut status = Status(0);
        status.set(Status::CARRY, true);
        assert!(status.is_set(Status::CARRY));
        status.set(Status::CARRY, false);
        assert!(!status.is_set(Status::CARRY));
    }

    #[test]
    fn setting_flag_is_idempotent() {
        let mut status = Status(0);
        status.set(Status::CARRY, true);
        assert!(status.is_set(Status::CARRY));
        status.set(Status::CARRY, true);
        assert!(status.is_set(Status::CARRY));
    }

    #[test]
    fn clearing_flag_is_idempotent() {
        let mut status = Status(0);
        status.set(Status::CARRY, true);
        assert!(status.is_set(Status::CARRY));
        status.set(Status::CARRY, false);
        assert!(!status.is_set(Status::CARRY));
        status.set(Status::CARRY, false);
        assert!(!status.is_set(Status::CARRY));
    }

    #[test]
    fn can_clear_all_used_flags() {
        let mut status = Status(0b1111_1111);
        status.set(Status::CARRY, false);
        status.set(Status::ZERO, false);
        status.set(Status::INTERRUPT_DISABLE, false);
        status.set(Status::DECIMAL_MODE, false);
        status.set(Status::BREAK_COMMAND, false);
        status.set(Status::OVERFLOW, false);
        status.set(Status::NEGATIVE, false);
        assert_eq!(status.0, Status::UNUSED);
    }
}
