use crate::apu::{SAMPLE_RATE, NES_CPU_CLOCK, MASTER_VOLUME};
use crate::apu::LENGTH_COUNTER_TABLE;

#[derive(Debug, Clone, Copy)]
pub struct Ch3Register {
    // $4008
    length: u8,
    linear_count_halt: bool,    // リニアカウンタ減算無効フラグ

    // $400A + $400B
    frequency: u16,

    // $400B (長さテーブルへのIndex)
    key_off_count: u8,

    // 内部で変化するカウンター類
    pub linear_counter: u8,     // リニアカウンタ (詳細な長さ制御)
    pub length_counter: u8,     // 長さカウンタ (大まかな長さ制御)
    linear_reload_flag: bool,   // リロードフラグ
    phase: f32,                 // 現在の波形の位置
}

impl Ch3Register {
    pub fn new() -> Self {
        Ch3Register {
            length: 0,
            linear_count_halt: false,
            frequency: 0,
            key_off_count: 0,
            linear_counter: 0,
            length_counter: 0,
            linear_reload_flag: false,
            phase: 0.0,
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x4008 => {
                self.length = data & 0x7F;
                self.linear_count_halt = (data & 0x80) != 0;
                self.linear_reload_flag = true;
            }
            0x4009 => {} // 未使用
            0x400A => {
                // タイマーの下位8bit更新
                self.frequency = (self.frequency & 0x0700) | data as u16;
            }
            0x400B => {
                // タイマーの上位3bit更新
                self.frequency = (self.frequency & 0x00FF) | (data as u16 & 0x07) << 8;

                // 長さカウンタのロード
                self.key_off_count = (data & 0xF8) >> 3;
                self.length_counter = LENGTH_COUNTER_TABLE[self.key_off_count as usize];

                // 副作用: 三角波の再トリガー時はリロードフラグをセットと位相リセット
                self.linear_reload_flag = true;
                self.phase = 0.0;
            }
            _ => panic!("can't be"),
        }
    }

    pub fn tick_length_counter(&mut self) {
        if self.linear_count_halt {
            return;
        }
        if self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    pub fn tick_linear_counter(&mut self) {
        if self.linear_reload_flag {
            self.linear_counter = self.length;
        } else if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }

        if !self.linear_count_halt {
            self.linear_reload_flag = false;
        }
    }

    pub fn next(&mut self) -> f32 {
        // カウンタが0なら音を止める（後述）
        if self.length_counter == 0 || self.linear_counter == 0 {
            return 0.0;
        }

        // 三角波の計算: 0.0~1.0 の phase を -1.0~1.0 の波形に変換
        let val = if self.phase <= 0.5 {
            self.phase
        } else {
            1.0 - self.phase
        };
        // 0.0~0.5 の範囲になっているので、中心をずらして4倍して -1.0~1.0 に正規化
        let x = (val - 0.25) * 4.0;

        // 次のサンプルのために位相を進める
        self.phase = (self.phase + self.hz() / SAMPLE_RATE) % 1.0;

        // 出力レベル
        return x * MASTER_VOLUME;
    }

    // 設定されたタイマー値から周波数(Hz)を算出
    fn hz(&self) -> f32 {
        NES_CPU_CLOCK / (32.0 * (self.frequency as f32 + 1.0))
    }
}
