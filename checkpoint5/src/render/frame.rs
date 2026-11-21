pub struct Frame {
    pub data: Vec<u8>,
    pub bg_opaque: Vec<bool>, // 追加：背景ピクセルが透明(0)でなければ true
}

impl Frame {
    const WIDTH: usize = 256 * 2;
    const HIGHT: usize = 240 ;

    pub fn new() -> Self {
        Frame {
            data: vec![0; (Frame::WIDTH) * (Frame::HIGHT) * 3],
            bg_opaque: vec![false; Frame::WIDTH * Frame::HIGHT],
        }
    }

    pub fn set_bg_opaque(&mut self, x: usize, y: usize, opaque: bool) {
        if x < Frame::WIDTH && y < Frame::HIGHT {
            self.bg_opaque[y * Frame::WIDTH + x] = opaque;
        }
    }

    pub fn is_bg_opaque(&self, x: usize, y: usize) -> bool {
        x < Frame::WIDTH && y < Frame::HIGHT && self.bg_opaque[y * Frame::WIDTH + x]
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, rgb: (u8, u8, u8)) {
        let base = y * 3 * Frame::WIDTH + x * 3;
        if base + 2 < self.data.len() {
            self.data[base] = rgb.0;
            self.data[base + 1] = rgb.1;
            self.data[base + 2] = rgb.2;
        }
    }
}
