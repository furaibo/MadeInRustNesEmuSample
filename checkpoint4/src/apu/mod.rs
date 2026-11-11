use sdl2::audio::{AudioQueue, AudioSpecDesired};

use registers::frame_sequencer::FrameSequencer;
use registers::status::StatusRegister;
use registers::ch1_register::Ch1Register;
use registers::ch2_register::Ch2Register;
use registers::ch3_register::Ch3Register;
use registers::ch4_register::Ch4Register;
use registers::ch5_register::Ch5Register;

pub mod registers;

// 定数定義
const SAMPLE_RATE: f32 = 44100.0;
const NES_CPU_CLOCK: f32 = 1_789_772.5;
const MASTER_VOLUME: f32 = 0.4; // 音量調整用

static LENGTH_COUNTER_TABLE: [u8; 32] = [
    0x0A, 0xFE, 0x14, 0x02, 0x28, 0x04, 0x50, 0x06, 0xA0, 0x08, 0x3C, 0x0A, 0x0E, 0x0C, 0x1A, 0x0E,
    0x0C, 0x10, 0x18, 0x12, 0x30, 0x14, 0x60, 0x16, 0xC0, 0x18, 0x48, 0x1A, 0x10, 0x1C, 0x20, 0x1E,
];

// 初期化関数
fn init_channel(sdl_context: &sdl2::Sdl) -> AudioQueue<f32> {
    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(SAMPLE_RATE as i32), // 44.1kHz
        channels: Some(1),              // モノラル(ファミコンはモノラルです。)
        samples: Some(256),             // バッファサイズ (適当な大きさ)
    };

    // デバイスを開く
    let device = audio_subsystem.open_queue(None, &desired_spec).unwrap();
    // 再生開始（これを忘れると音が出ません）
    device.resume();

    return device;
}

pub struct NesAPU {
    frame_sequencer: FrameSequencer,
    status: StatusRegister,
    ch1_register: Ch1Register,
    ch2_register: Ch2Register,
    ch3_register: Ch3Register,
    ch4_register: Ch4Register,
    ch5_register: Ch5Register,
    device: AudioQueue<f32>, // オーディオデバイスへのハンドル
    cycles: usize, // CPUサイクル積算用
    counter: usize, // 現在のステップ数保持用
}

impl NesAPU {
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let device = init_channel(sdl_context);

        NesAPU {
            frame_sequencer: FrameSequencer::new(),
            status: StatusRegister::new(),
            ch1_register: Ch1Register::new(),
            ch2_register: Ch2Register::new(),
            ch3_register: Ch3Register::new(),
            ch4_register: Ch4Register::new(),
            ch5_register: Ch5Register::new(),
            device: device,
            cycles: 0,
            counter: 0,
        }
    }

    pub fn write_frame_sequencer(&mut self, data: u8) {
        // フレームシーケンサーに書き込まれると、ステップ数が初期化されます。
        self.frame_sequencer.update(data);
        self.cycles = 0;
        self.counter = 0;
    }

    // $4015 Write
    pub fn write_status(&mut self, data: u8) {
        self.status.update(data);
        // DMCの実装時に追加処理が必要になります。
    }

    // $4015 Read
    pub fn read_status(&mut self) -> u8 {
        let mut res = self.status.bits();

        // Bit 0-3: 長さカウンタが残っているか？

        // あとでch3_register本体を実装しますが、先に書いちゃいます。
        res = res
            | (if self.ch3_register.length_counter == 0 {
            0
        } else {
            1
        } << 2);

        // 重要な副作用: フレームIRQフラグをクリアする
        self.status.remove(StatusRegister::ENABLE_FRAME_IRQ);

        res
    }

    // CPUへのIRQ通知
    pub fn frame_irq(&self) -> bool {
        self.status.contains(StatusRegister::ENABLE_FRAME_IRQ)
    }

    pub fn tick(&mut self, cycles: u8, prg_rom: &[u8]) {
        self.cycles += cycles as usize;

        // 約7457サイクルごとにステップを進める
        let interval = 7457;
        if self.cycles > interval {
            self.cycles -= interval;
            self.counter += 1;

            // 音を鳴らす
            self.add_buffer(cycles, prg_rom);

            match self.frame_sequencer.mode() {
                4 => {
                    // - - - f      60 Hz
                    // - l - l     120 Hz
                    // e e e e     240 Hz
                    if self.counter == 2 || self.counter == 4 {
                        // 長さカウンタとスイープユニットのクロック生成
                        self.send_length_counter_tick();
                        self.send_sweep_tick();
                    }
                    if self.counter == 4 {
                        // 割り込みフラグセット
                        self.counter = 0;
                        if self.frame_sequencer.irq() {
                            self.status.insert(StatusRegister::ENABLE_FRAME_IRQ);
                        }
                    }
                    // エンベロープと三角波の線形カウンタのクロック生成
                    self.send_envelope_tick();
                }
                5 => {
                    // - - - - -   (割り込みフラグはセットしない)
                    // - l - - l   96 Hz
                    // e e e - e  192 Hz

                    if self.counter == 1 || self.counter == 4 {
                        // 長さカウンタとスイープユニットのクロック生成
                        self.send_length_counter_tick();
                        self.send_sweep_tick();
                    }
                    if self.counter != 3 {
                        // エンベロープと三角波の線形カウンタのクロック生成
                        self.send_envelope_tick();
                    }
                    if self.counter == 5 {
                        self.counter = 0;
                    }
                }
                _ => panic!("can't be"),
            }
        }
    }

    // tick内で定期的に呼ばれる想定
    fn add_buffer(&mut self, cycles: u8, prg_rom: &[u8]) {
        // 1step分(1/240秒)で必要なサンプル数
        let must_add = SAMPLE_RATE / 240.0;
        // バッファが空になって音が途切れないように最低サイズを定義。(5フレーム分)
        let buffer_min_size = SAMPLE_RATE * (5.0 / 60.0);
        // デバイスの未再生分がどれだけあるかを取得。
        let buffer_size = self.device.size() as f32 / 4.0; // f32=4byteなので

        // 今回追加するサンプル数を計算。
        let add_buffer_size = if buffer_min_size > buffer_size {
            buffer_min_size - buffer_size
        } else {
            must_add
        };

        let mut buffer = vec![0.0; add_buffer_size as usize];

        // サンプルを生成して埋める
        for sample in buffer.iter_mut() {
            *sample = 0.0;

            if self.status.contains(StatusRegister::ENABLE_1CH) {
                *sample += self.ch1_register.next();
            }
            if self.status.contains(StatusRegister::ENABLE_2CH) {
                *sample += self.ch2_register.next();
            }
            if self.status.contains(StatusRegister::ENABLE_3CH) {
                *sample += self.ch3_register.next();
            }
            if self.status.contains(StatusRegister::ENABLE_4CH) {
                *sample += self.ch4_register.next();
            }
            if self.status.contains(StatusRegister::ENABLE_5CH) {
                *sample += self.ch5_register.next(cycles, prg_rom);
            }
        }

        // IRQ発生の確認(5chのみ)
        if self.ch5_register.irq_flag {
            self.status.insert(StatusRegister::ENABLE_FRAME_IRQ);
        }

        // SDLに送信
        self.device.queue_audio(&buffer).unwrap();
    }

    pub fn write1ch(&mut self, addr: u16, value: u8) {
        self.ch1_register.write(addr, value);
    }

    pub fn write2ch(&mut self, addr: u16, value: u8) {
        self.ch2_register.write(addr, value);
    }

    pub fn write3ch(&mut self, addr: u16, value: u8) {
        self.ch3_register.write(addr, value);
    }

    pub fn write4ch(&mut self, addr: u16, value: u8) {
        self.ch4_register.write(addr, value);
    }

    pub fn write5ch(&mut self, addr: u16, value: u8) {
        self.ch5_register.write(addr, value);
    }

    // エンベロープと三角波の線形カウンタのクロック生成
    fn send_envelope_tick(&mut self) {
        self.ch1_register.tick_envelope();
        self.ch2_register.tick_envelope();
        self.ch3_register.tick_linear_counter();
        self.ch4_register.tick_envelope();
    }

    // 長さカウンタのクロック生成
    fn send_length_counter_tick(&mut self) {
        self.ch1_register.tick_length_counter();
        self.ch2_register.tick_length_counter();
        self.ch3_register.tick_length_counter();
        self.ch4_register.tick_length_counter();
    }

    // スイープユニットのクロック生成
    fn send_sweep_tick(&mut self) {
        self.ch1_register.tick_sweep();
        self.ch2_register.tick_sweep();
    }
}