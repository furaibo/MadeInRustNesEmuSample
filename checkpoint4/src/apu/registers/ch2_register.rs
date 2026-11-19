use crate::apu::{SAMPLE_RATE, NES_CPU_CLOCK, MASTER_VOLUME};
use crate::apu::LENGTH_COUNTER_TABLE;

#[derive(Debug, Clone, Copy)]
pub struct Ch2Register {
    // $4004
    volume: u8,
    envelope_flag: bool,
    key_off_counter_flag: bool,
    duty: u8,

    // $4005
    sweep_change_amount: u8,
    sweep_direction: u8,
    sweep_timer_count: u8,
    sweep_enabled: bool,

    // $4006 + $4007
    frequency: u16,
    key_off_count: u8,

    // 内部計算用
    phase: f32,
    pub length_counter: u8,
    envelope_counter: u8,
    envelope_division_period: u8,
    sweep_counter: u8,
}

// デューティ比
pub const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0], // 12.5%
    [0, 1, 1, 0, 0, 0, 0, 0], // 25%
    [0, 1, 1, 1, 1, 0, 0, 0], // 50%
    [1, 0, 0, 1, 1, 1, 1, 1], // 75%
];

impl Ch2Register {
    pub fn new() -> Self {
        Ch2Register {
            volume: 0,
            envelope_flag: false,
            key_off_counter_flag: false,
            duty: 0,

            sweep_change_amount: 0,
            sweep_direction: 0,
            sweep_timer_count: 0,
            sweep_enabled: false,

            frequency: 0,
            key_off_count: 0,

            phase: 0.0,
            length_counter: 0,
            envelope_counter: 0x0F,
            envelope_division_period: 1,
            sweep_counter: 0,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x4004 => {
                self.volume = value & 0x0F;
                self.envelope_flag = (value & 0x10) == 0;
                self.key_off_counter_flag = (value & 0x20) == 0;
                self.duty = (value & 0xC0) >> 6;
            }
            0x4005 => {
                self.sweep_change_amount = value & 0x07;
                self.sweep_direction = (value & 0x08) >> 3;
                self.sweep_timer_count = (value & 0x70) >> 4;
                self.sweep_enabled = (value & 0x80) != 0;
            }
            0x4006 => {
                self.frequency = (self.frequency & 0x0700) | value as u16;
            }
            0x4007 => {
                self.frequency = (self.frequency & 0x00FF) | (value as u16 & 0x07) << 8;
                self.key_off_count = (value & 0xF8) >> 3;
                self.length_counter = LENGTH_COUNTER_TABLE[self.key_off_count as usize];

                // リセット処理
                self.envelope_counter = 0x0F;
                self.envelope_division_period = self.volume + 1;
                self.sweep_counter = 0;
                self.phase = 0.0;
            }
            _ => panic!("can't be"),
        }
    }

    fn hz(&self) -> f32 {
        if self.frequency == 0 {
            return 0.0;
        }
        NES_CPU_CLOCK / (16.0 * (self.frequency as f32 + 1.0))
    }

    fn volume(&self) -> f32 {
        // エンベロープ有効なら内部カウンタ、無効なら固定音量(volume)を使用
        (if self.envelope_flag {
            self.envelope_counter
        } else {
            self.volume
        }) as f32
            / 15.0
    }

    pub fn next(&mut self) -> f32 {
        // 長さカウンタが0なら無音
        if self.length_counter == 0 {
            return 0.0;
        }

        let duty = DUTY_TABLE[self.duty as usize];
        let step = ((self.phase * 8.0).floor() as usize) % 8;

        // 波形が 0 ならマイナス方向、1 ならプラス方向へ振幅させる
        let x = if duty[step] == 0 {
            -self.volume()
        } else {
            self.volume()
        } * MASTER_VOLUME;

        let hz = self.hz();
        if hz != 0.0 {
            self.phase = (self.phase + hz / SAMPLE_RATE) % 1.0;
        }
        return x;
    }

    pub fn tick_envelope(&mut self) {
        self.envelope_division_period -= 1;
        if self.envelope_division_period != 0 {
            return;
        }

        // 周期が来たら音量を更新
        if self.envelope_counter != 0 {
            self.envelope_counter -= 1;
        } else if self.envelope_counter == 0 {
            // ループフラグ(key_off_counter_flagの反転)が有効なら音量を最大に戻す
            if self.key_off_counter_flag {
                self.envelope_counter = 0x0F;
                self.envelope_division_period = self.volume + 1;
            }
        }
        self.envelope_division_period = self.volume + 1;
    }

    pub fn tick_sweep(&mut self) {
        // チャンネルの長さカウンタがゼロなら処理しない
        if self.length_counter == 0 {
            return;
        }

        self.sweep_counter += 1;
        if self.sweep_counter < (self.sweep_timer_count + 1) {
            return;
        }
        self.sweep_counter = 0;

        // スイープが無効なら処理しない
        if !self.sweep_enabled {
            return;
        }
        if self.sweep_change_amount == 0 {
            return;
        }

        if self.sweep_direction == 0 {
            // しり下がりモード: 周波数値を増やす（音程を下げる）
            // 新しい周期 = 周期 + (周期 >> N)
            self.frequency = self.frequency + (self.frequency >> self.sweep_change_amount);
        } else {
            // しり上がりモード: 周波数値を減らす（音程を上げる）
            // 新しい周期 = 周期 - (周期 >> N)
            self.frequency = self.frequency - (self.frequency >> self.sweep_change_amount);
        }

        // 周波数が範囲外になったらミュートする（長さカウンタを0にする）
        if self.frequency < 0x08 || self.frequency > 0x7FF {
            self.key_off_count = 0;
            self.length_counter = 0;
        }
    }

    pub fn tick_length_counter(&mut self) {
        // カウンタ停止フラグが立っていなければ減算
        if !self.key_off_counter_flag {
            return;
        }
        if self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }
}
