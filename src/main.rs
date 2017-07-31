extern crate sdl2;

use std::env;
use sdl2::event::Event;
use sdl2::rect::Rect;
use sdl2::keyboard::Keycode;
use sdl2::surface::Surface;
use std::time::Duration;
use std::path::Path;
use std::thread;
use std::process;

const DELTA_ANGLE: f64 = 0.5;
const HELP: &'static str = r#"
SYNOPSIS
    poxel-viewer [flags] [bmp file]

FLAGS
    -w [px], --width [px]        : Set the width of each layer to [px] pixels
    -s [px], --step [px]         : Set the step in Y between every layer to [px] pixels
    -z [factor], --zoom [factor] : Zoom in by [factor]
    -h, --help                   : Print this help page
    -R, --disable-rotation       : Disable automatic rotation

CONTROLS
    Q, escape : Quit the program
    J, left   : Rotate left
    K, right  : Rotate right
    +         : Zoom in
    -         : Zoom out
    R         : Reload file
    space     : Toggle rotation
"#;

pub fn main() {
    let mut args = env::args().skip(1);
    let mut sprite_width: u32 = 64;
    let mut rotate = true;
    let mut step = 4;
    let mut path = String::new();
    let mut zoom = 1;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-w" | "--width" => sprite_width = args
                .next()
                .unwrap_or_else(|| {
                    println!("No number following --width.");
                    process::exit(1);
                })
                .parse()
                .unwrap_or_else(|_| {
                    println!("What follows --width should be a valid integer.");
                    process::exit(1);
                }),
            "-s" | "--step" => step = args
                .next()
                .unwrap_or_else(|| {
                    println!("No number following --step.");
                    process::exit(1);
                })
                .parse()
                .unwrap_or_else(|_| {
                    println!("What follows --step should be a valid integer.");
                    process::exit(1);
                }),
            "-z" | "--zoom" => zoom = args
                .next()
                .unwrap_or_else(|| {
                    println!("No number following --zoom.");
                    process::exit(1);
                })
                .parse()
                .unwrap_or_else(|_| {
                    println!("What follows --zoom should be a valid integer.");
                    process::exit(1);
                }),
            "-R" | "--disable-rotations" => rotate = false,
            "-h" | "--help" => return println!("{}", HELP),
            x if !x.starts_with("-") => path = x.into(),
            x => {
                println!("Unknown flag {}, see --help.", x);
                process::exit(1);
            },
        }
    }

    if path.is_empty() {
        println!("No path is provided, see --help.");
        process::exit(1);
    }

    let sdl_context = sdl2::init().unwrap();

    let mut sprites_surface = Surface::load_bmp(Path::new(&path))
        .unwrap_or_else(|_| {
            println!("Could not load BMP at '{}'.", path);
            process::exit(1);
        });

    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Poxel Viewer", 200, 200)
        .position_centered()
        .allow_highdpi()
        .resizable()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let tc = canvas.texture_creator();

    let mut sprites = tc.create_texture_from_surface(sprites_surface).unwrap();
    let sprite_height = sprites.query().height;

    let mut event_pump = sdl_context.event_pump().unwrap();
    let layers = sprites.query().width / sprite_width;

    let mut angle = 0.;

    'running: loop {
        canvas.clear();
        let (width, height) = canvas.output_size().unwrap();

        for layer in 0..layers as i32 + 1 {
            for sublayer in 0..zoom as i32 {
                canvas.copy_ex(
                    &sprites,
                    Rect::new(layer * sprite_width as i32, 0, sprite_width, sprite_height),
                    Rect::new(
                        width as i32 / 2 - sprite_width as i32 / 2 * zoom as i32,
                        height as i32 / 2 - (sprite_height as i32 / 2 + layer * step) * zoom as i32 - sublayer * step,
                        sprite_width * zoom,
                        sprite_height * zoom,
                    ),
                    angle,
                    None,
                    false,
                    false,
                ).unwrap();
            }
        }
        canvas.present();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..}
                    | Event::KeyDown { keycode: Some(Keycode::Escape), .. }
                    | Event::KeyDown { keycode: Some(Keycode::Q), .. }
                    => break 'running,
                Event::KeyDown { keycode: Some(Keycode::J), .. }
                    | Event::KeyDown { keycode: Some(Keycode::Left), .. }
                    => angle += 20.0 * DELTA_ANGLE,
                Event::KeyDown { keycode: Some(Keycode::K), .. }
                    | Event::KeyDown { keycode: Some(Keycode::Right), .. }
                    => angle -= 16.0 * DELTA_ANGLE,
                Event::KeyDown { keycode: Some(Keycode::R), .. } => {
                    sprites_surface = Surface::load_bmp(Path::new(&path))
                        .unwrap_or_else(|_| {
                            println!("Could not load BMP at '{}'.", path);
                            process::exit(1);
                        });
                    sprites = tc.create_texture_from_surface(sprites_surface).unwrap();
                },
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => rotate = !rotate,
                Event::KeyDown { keycode: Some(Keycode::Equals), .. } => zoom += 1,
                Event::KeyDown { keycode: Some(Keycode::Minus), .. } => zoom -= 1,
                _ => {}
            }
        }

        if rotate { angle += DELTA_ANGLE; }

        thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
