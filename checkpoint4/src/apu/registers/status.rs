bitflags! {
    pub struct StatusRegister: u8 {
        const ENABLE_1CH       = 0b0000_0001;
        const ENABLE_2CH       = 0b0000_0010;
        const ENABLE_3CH       = 0b0000_0100;
        const ENABLE_4CH       = 0b0000_1000;
        const ENABLE_5CH       = 0b0001_0000;

        const ENABLE_FRAME_IRQ = 0b0100_0000;
        const ENABLE_DMC_IRQ   = 0b1000_0000;
    }
}

impl StatusRegister {
    pub fn new() -> Self {
        StatusRegister::empty()
    }

    pub fn update(&mut self, data: u8) {
        *self.0.bits_mut() = data;
    }
}
