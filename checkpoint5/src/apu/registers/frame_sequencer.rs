// フレームシーケンサーレジスタ
// bitfrags を利用し指定のビットを取得できるようにする
bitflags! {
    pub struct FrameSequencer: u8 {
        const DISABLE_IRQ    = 0b0100_0000;
        const SEQUENCER_MODE = 0b1000_0000;
    }
}

impl FrameSequencer {
    pub fn new() -> Self {
        FrameSequencer::empty()
    }

    // モード取得
    pub fn mode(&self) -> u8 {
        if self.contains(FrameSequencer::SEQUENCER_MODE) {
            5
        } else {
            4
        }
    }

    // IRQフラグ
    pub fn irq(&self) -> bool {
        !self.contains(FrameSequencer::DISABLE_IRQ)
    }

    // レジスタ内容更新
    pub fn update(&mut self, data: u8) {
        *self.0.bits_mut() = data;
    }
}