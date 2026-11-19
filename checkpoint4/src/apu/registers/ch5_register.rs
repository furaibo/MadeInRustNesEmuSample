use crate::apu::{SAMPLE_RATE, NES_CPU_CLOCK, MASTER_VOLUME};

// 周波数テーブル
// NTSC版のDMC期間テーブル（CPUサイクル単位）
const DMC_RATE_TABLE: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214,
    190, 160, 142, 128, 106,  84,  72,  54
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ch5Register {
    // $4010
    irq_enabled: bool,
    pub irq_flag: bool,
    loop_flag: bool,
    rate_index: u8,
    init_rate_counter: u16,

    // $4011
    output_level: u8,

    // $4012
    start_addr: u8,

    // $4013
    byte_count: u8,

    // 内部計算用
    data: u8,
    rate_counter: u16,
    sample_addr: u16,
    pub bit_remaining: u32,
}

impl Ch5Register {
    pub fn new() -> Self {
        Ch5Register {
            irq_enabled: false,
            irq_flag: false,
            loop_flag: false,
            rate_index: 0,
            init_rate_counter: 0,

            output_level: 0,
            start_addr: 0,
            byte_count: 0,

            rate_counter: 0,
            data: 0,
            sample_addr: 0xC000,
            bit_remaining: (0 * 8) as u32 * 0x10 + 1,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x4010 => {
                self.irq_enabled = value & 0x80 != 0;
                self.irq_flag = false;
                self.loop_flag = value & 0x40 != 0;
                self.rate_index = value & 0x0F;
                self.init_rate_counter = DMC_RATE_TABLE[self.rate_index as usize];
            }
            0x4011 => {
                self.output_level = value & 0x7F;
            }
            0x4012 => {
                self.start_addr = value;
            }
            0x4013 => {
                self.byte_count = value;
                if self.bit_remaining == 0 {
                    self.reload_sample_bytes();
                }
            }
            _ => panic!("can't be"),
        }
    }

    // 音源再生のための初期化処理(ループ再生時にも呼び出す)
    pub fn reload_sample_bytes(&mut self) {
        self.rate_counter = self.init_rate_counter;
        self.sample_addr = self.start_addr as u16 * 0x40 + 0xC000;
        self.bit_remaining = (self.byte_count as u32 * 16 + 1) * 8;
        self.data = 0;
    }

    // PRG ROMデータを読み込む(5ch専用)
    fn read_prg_rom(&self, prg_rom: &[u8], mut addr: u16) -> u8 {
        addr -= 0x8000;
        if prg_rom.len() == 0x4000 && addr >= 0x4000 {
            addr = addr % 0x4000;
        }
        prg_rom[addr as usize]
    }

    pub fn next(&mut self, cycles: u8, prg_rom: &[u8]) -> f32 {
        let mut x = 0.0;

        // 指定のサイクル数を待って音の再生
        if self.rate_counter > cycles as u16 {
            self.rate_counter -= cycles as u16;
        } else {
            let remainder = cycles as u16 - self.rate_counter;
            self.rate_counter = self.init_rate_counter.wrapping_sub(remainder);

            if self.bit_remaining > 0 {
                // バイト境界でのメモリ読み込み
                if self.bit_remaining & 0x0007 == 0 {
                    self.data = self.read_prg_rom(prg_rom, self.sample_addr);

                    if self.sample_addr == 0xFFFF {
                        self.sample_addr = 0x8000;
                    } else {
                        self.sample_addr += 1;
                    }
                }

                // シフト＆出力レベル更新
                if self.data & 0x01 != 0 {
                    if self.output_level < 126 {
                        self.output_level += 2
                    }
                } else {
                    if self.output_level > 1 {
                        self.output_level -= 2
                    }
                }
                self.data = self.data >> 1;
                self.bit_remaining -= 1;

            } else {
                // 再生終了、またはループ時の処理
                if self.loop_flag {
                    self.reload_sample_bytes();
                } else {
                    if self.irq_enabled {
                        // Note: IRQ発生フラグの有効化
                        self.irq_flag = true;
                    }
                }
            }
        }

        // 出力レベルの決定
        // Note:
        //   64を引いて64で割ることで波形の中央を0.0に補正し、
        //   他の矩形波や三角波とミックスした際に
        //   直流成分(ポップノイズの原因)が乗らない綺麗な波形に整える
        x = ((self.output_level as f32 - 64.0) / 64.0) * MASTER_VOLUME;

        return x;
    }
}
