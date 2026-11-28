pub mod bus;
pub mod cartridge;
pub mod cpu;
pub mod joypad;
pub mod opcodes;
pub mod ppu;
pub mod render;
pub mod trace;
pub mod apu;

use bus::Bus;
use cartridge::Rom;
use cpu::CPU;
use ppu::NesPPU;
use render::frame::Frame;
// use trace::trace;
use apu::NesAPU;

use std::path::PathBuf;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use clap::Parser;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate bitflags;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    file: Option<PathBuf>,

    #[arg(short, long)]
    scale: Option<f32>,
}

const DEFAULT_SCALE_VALUE: f32 = 2.0;
const NES_FRAME_WIDTH: u32 = 256;
const NES_FRAME_HEIGHT: u32 = 240;
const NES_FRAME_DURATION: Duration = Duration::from_nanos(16_639_267);

fn main() {
    // get file path
    let args = Args::parse();
    let path = args.file.unwrap_or_else(|| PathBuf::from("rom/test/color_test.nes"));

    // get scale
    let scale = args.scale.unwrap_or(DEFAULT_SCALE_VALUE);

    // load the game
    //let bytes: Vec<u8> = std::fs::read("rom/pacman.nes").unwrap();
    let bytes: Vec<u8> = std::fs::read(path).unwrap();
    let rom = Rom::new(&bytes).unwrap();

    // init sdl2
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Game Window", (NES_FRAME_WIDTH as f32 * scale) as u32, (NES_FRAME_WIDTH as f32 * scale) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(scale, scale).unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, NES_FRAME_WIDTH, NES_FRAME_HEIGHT)
        .unwrap();

    let mut frame = Frame::new();

    let mut key_map = HashMap::new();
    key_map.insert(Keycode::Down, joypad::JoypadButton::DOWN);
    key_map.insert(Keycode::Up, joypad::JoypadButton::UP);
    key_map.insert(Keycode::Right, joypad::JoypadButton::RIGHT);
    key_map.insert(Keycode::Left, joypad::JoypadButton::LEFT);
    key_map.insert(Keycode::Space, joypad::JoypadButton::SELECT);
    key_map.insert(Keycode::Return, joypad::JoypadButton::START);
    key_map.insert(Keycode::A, joypad::JoypadButton::BUTTON_A);
    key_map.insert(Keycode::S, joypad::JoypadButton::BUTTON_B);

    let apu = NesAPU::new(&sdl_context);
    let mut next_frame = Instant::now();

    // run the game cycle
    let bus = Bus::new(rom, apu, move |ppu: &NesPPU, joypad: &mut joypad::Joypad| {
        render::render(ppu, &mut frame);
        texture.update(None, &frame.data, 256 * 2 * 3).unwrap();

        canvas.copy(&texture, None, None).unwrap();

        // フレーム間隔を60Hzに揃える
        next_frame += NES_FRAME_DURATION;
        let now = Instant::now();
        if next_frame > now {
            let remain = next_frame - now;
            // sleep は精度が粗いので、手前まで sleep して残りは空回しで待つ
            if remain > Duration::from_millis(2) {
                std::thread::sleep(remain - Duration::from_millis(2));
            }
            while Instant::now() < next_frame {
                std::hint::spin_loop();
            }
        } else if now - next_frame > NES_FRAME_DURATION * 5 {
            // 大きく遅れたとき(ウィンドウ移動など)は、遅れを取り戻さず仕切り直す
            next_frame = now;
        }

        canvas.present();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => std::process::exit(0),

                Event::KeyDown { keycode, .. } => {
                    if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        joypad.set_button_pressed_status(*key, true);
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                        joypad.set_button_pressed_status(*key, false);
                    }
                }

                _ => { /* do nothing */ }
            }
        }
    });

    let mut cpu = CPU::new(bus);
    cpu.reset();
    cpu.run();
    /*
    cpu.run_with_callback(|cpu| {
        println!("{}", trace(cpu));
    });
    */
}
