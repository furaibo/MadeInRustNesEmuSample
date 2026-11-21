use crate::apu::{SAMPLE_RATE, NES_CPU_CLOCK, MASTER_VOLUME};
use crate::apu::LENGTH_COUNTER_TABLE;

// ノイズ周波数テーブル (NTSC)
// インデックスに対応する「タイマー周期（CPUサイクル数）」を保持
static NOISE_TABLE: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254,
    380, 508, 762, 1016, 2034, 4068,
];

#[derive(Debug, Clone, Copy, PartialEq)]
enum NoiseKind {
    Long,  // ロングモード
    Short, // ショートモード
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ch4Register {
    // $400C
    volume: u8,
    envelope_flag: bool,
    key_off_counter_flag: bool,

    // $400E
    frequency: u8,
    kind: NoiseKind,

    // $400F
    key_off_count: u8,

    // 内部計算用
    phase: f32,
    random: u16, // 重要: 乱数生成用シフトレジスタ
    pub length_counter: u8,
    envelope_counter: u8,
    envelope_division_period: u8,
}

impl Ch4Register {
    pub fn new() -> Self {
        Ch4Register {
            volume: 0,
            envelope_flag: false,
            key_off_counter_flag: false,
            frequency: 0,
            kind: NoiseKind::Long,
            key_off_count: 0,
            phase: 0.0,
            random: 1,   // 注意: 非ゼロ値で初期化すること
            length_counter: 0,
            envelope_counter: 0x0F,
            envelope_division_period: 1,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x400C => {
                self.volume = value & 0x0F;
                self.envelope_flag = (value & 0x10) == 0;
                self.key_off_counter_flag = (value & 0x20) != 0;
            }
            0x400E => {
                self.frequency = value & 0x0F;
                self.kind = match value & 0x80 {
                    0 => NoiseKind::Long,
                    _ => NoiseKind::Short,
                };
            }
            0x400F => {
                self.key_off_count = (value & 0xF8) >> 3;
                self.length_counter = LENGTH_COUNTER_TABLE[self.key_off_count as usize];

                // 書き込み時のリセット処理
                self.phase = 0.0;
                self.random = 1; // ここでリセットしないと音が変わらないことがある
                self.envelope_counter = 0x0F;
                self.envelope_division_period = self.volume + 1;
            }
            _ => panic!("can't be"),
        }
    }

    // テーブルを使って周波数(Hz)に変換
    fn hz(&self) -> f32 {
        NES_CPU_CLOCK / NOISE_TABLE[self.frequency as usize] as f32
    }

    // LFSRによる疑似乱数生成
    fn next_random(&mut self) {
        let bit1 = (self.random >> (if self.kind == NoiseKind::Long { 1 } else { 6 })) & 0x01;
        let bit2 = self.random & 0x01;
        self.random = (self.random >> 1) | (bit1 ^ bit2) << 14;
    }

    // 音量計算 (1ch/2chと同様)
    fn volume(&self) -> f32 {
        (if self.envelope_flag {
            self.envelope_counter
        } else {
            self.volume
        }) as f32
            / 15.0
    }

    pub fn next(&mut self) -> f32 {
        // 現在の乱数の0ビット目を見て出力を決定
        // 0なら出力、1なら無音
        let mut x =
            if (self.random & 0x01) != 0 { 0.0 } else { 1.0 } * self.volume() * MASTER_VOLUME;

        if self.length_counter == 0 {
            x = 0.0;
        }

        let last_phase = self.phase;

        // 1サンプルで進むべき位相の量を計算
        // 高音の場合、この値は 1.0 を大きく超える
        let mut add = self.hz() / SAMPLE_RATE;
        self.phase = (self.phase + add) % 1.0;

        // add が 1.0 を超えている場合、その回数分だけ乱数生成器(next_random)を回す
        // これにより、サンプリングレート間の乱数の変化をエミュレートする
        loop {
            if add < 1.0 {
                break;
            }
            add -= 1.0;
            self.next_random();
        }

        // 最後に、端数の分で位相またぎが発生していたらもう1回更新
        if self.phase < last_phase {
            self.next_random();
        }

        return x;
    }

    // 長さカウンタ更新(共通)
    pub fn tick_length_counter(&mut self) {
        if self.key_off_counter_flag {
            return;
        }
        if self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }

    // エンベロープ更新(共通)
    pub fn tick_envelope(&mut self) {
        self.envelope_division_period -= 1;
        if self.envelope_division_period != 0 {
            return;
        }

        if self.envelope_counter != 0 {
            self.envelope_counter -= 1;
        } else if self.envelope_counter == 0 {
            if self.key_off_counter_flag {
                self.envelope_counter = 0x0F;
                self.envelope_division_period = self.volume + 1;
            }
        }
        self.envelope_division_period = self.volume + 1;
    }
}
