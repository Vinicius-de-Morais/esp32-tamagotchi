use esp_hal::peripherals::TIMG0;
use crate::peripherals::timer::TimerPeripherals;

pub struct Factory;

impl<'a> Factory {
    pub fn create_timer_group0(timer_peripherals: TimerPeripherals) -> esp_hal::timer::timg::TimerGroup<'a, TIMG0<'a>> {
        esp_hal::timer::timg::TimerGroup::new(timer_peripherals.timer0)
    }
}