use esp_hal::peripherals::TIMG0;


pub struct TimerPeripherals {
    pub timer0: TIMG0<'static>,
}

impl TimerPeripherals {
    pub fn new(timer0: TIMG0<'static>) -> Self {
        TimerPeripherals { timer0 }
    }
}