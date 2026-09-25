use crate::{bus::Bus, cpu::Cpu};

pub struct Console<T: Bus> {
    cpu: Cpu,
    bus: T,
}
