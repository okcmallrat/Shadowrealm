#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
use minifb::{Key, KeyRepeat, Scale, ScaleMode, Window, WindowOptions};
use shadowrealm::{game::Game, input, Rom};
use std::{
    error::Error,
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};
#[cfg(windows)]
fn report_error(message: &str) {
    #[link(name = "user32")]
    unsafe extern "system" {
        fn MessageBoxW(
            window: *mut std::ffi::c_void,
            text: *const u16,
            caption: *const u16,
            flags: u32,
        ) -> i32;
    }
    let text: Vec<u16> = message.encode_utf16().chain(Some(0)).collect();
    let title: Vec<u16> = "Shadowrealm".encode_utf16().chain(Some(0)).collect();
    unsafe {
        MessageBoxW(std::ptr::null_mut(), text.as_ptr(), title.as_ptr(), 0x10);
    }
}
#[cfg(not(windows))]
fn report_error(message: &str) {
    eprintln!("{message}");
}
fn main() {
    if let Err(error) = run() {
        report_error(&format!("Unable to start Shadowrealm: {error}\n\nPlace Shadowrun.bin beside Shadowrealm.exe, or pass its path on the command line."));
        std::process::exit(1);
    }
}
fn run() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args_os().skip(1);
    let mut positional = vec![];
    let mut solo = false;
    let mut funds = None;
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--solo") => solo = true,
            Some("--nuyen") => {
                let amount: u32 = args
                    .next()
                    .ok_or("--nuyen needs an amount")?
                    .to_string_lossy()
                    .parse()?;
                if amount > i32::MAX as u32 {
                    return Err("--nuyen must be between 0 and 2147483647".into());
                }
                funds = Some(amount);
            }
            Some(flag) if flag.starts_with("--") => {
                return Err(format!("Unknown option {flag}").into())
            }
            _ => positional.push(arg),
        }
    }
    if positional.len() > 2 {
        return Err("Usage: Shadowrealm.exe [ROM] [MAP] [--solo] [--nuyen AMOUNT]".into());
    }
    let rom_path = if let Some(path) = positional.first() {
        PathBuf::from(path)
    } else {
        std::env::current_exe()?
            .parent()
            .ok_or("executable directory unavailable")?
            .join("Shadowrun.bin")
    };
    let map_id: usize = positional
        .get(1)
        .map(|v| v.to_string_lossy().parse())
        .transpose()?
        .unwrap_or(0);
    let rom = fs::read(rom_path)?;
    let parsed = Rom::parse(&rom)?;
    if parsed.checksum() != parsed.stored_checksum() {
        return Err("ROM checksum mismatch".into());
    }
    let templates: &[usize] = if solo { &[0] } else { &[0, 3, 10] };
    let load_session = || -> Result<Game, Box<dyn Error>> {
        let mut g = Game::load(&rom, map_id, templates)?;
        if let Some(amount) = funds {
            g.nuyen = amount;
        }
        Ok(g)
    };
    let mut game = load_session()?;
    let mut window = Window::new(
        "Shadowrealm — Exploration build",
        320,
        224,
        WindowOptions {
            scale: Scale::X2,
            scale_mode: ScaleMode::AspectRatioStretch,
            resize: true,
            ..WindowOptions::default()
        },
    )?;
    window.set_target_fps(60);
    let mut paused = false;
    let mut buffer = vec![0u32; 320 * 224];
    let mut accumulator = Duration::ZERO;
    let mut last = Instant::now();
    let step = Duration::from_nanos(1_000_000_000 / 60);
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let now = Instant::now();
        accumulator += (now - last).min(Duration::from_millis(100));
        last = now;
        if window.is_key_pressed(Key::Space, KeyRepeat::No) {
            paused = !paused;
            accumulator = Duration::ZERO;
        }
        if window.is_key_pressed(Key::Tab, KeyRepeat::No) {
            let selected = (game.selected + 1) % game.party.len();
            game.select(&rom, selected)?;
        }
        if window.is_key_pressed(Key::R, KeyRepeat::No) {
            game = load_session()?;
        }
        let mut held = 0;
        for (keys, button) in [
            ([Key::Up, Key::W], input::UP),
            ([Key::Down, Key::S], input::DOWN),
            ([Key::Left, Key::A], input::LEFT),
            ([Key::Right, Key::D], input::RIGHT),
        ] {
            if keys.iter().any(|key| window.is_key_down(*key)) {
                held |= button;
            }
        }
        if game.dialogue.is_none() && window.is_key_down(Key::T) {
            held |= input::C;
        }
        if game.dialogue.is_none() && window.is_key_down(Key::F) {
            held |= input::A;
        }
        if game.dialogue.is_some() && (window.is_key_down(Key::Enter) || window.is_key_down(Key::Z))
        {
            held |= input::A;
        }
        if paused {
            accumulator = Duration::ZERO;
        } else {
            while accumulator >= step {
                game.update(&rom, held)?;
                accumulator -= step;
            }
        }
        let pixels = game.render(&rom)?;
        for (out, rgb) in buffer.iter_mut().zip(pixels.chunks_exact(3)) {
            *out = (u32::from(rgb[0]) << 16) | (u32::from(rgb[1]) << 8) | u32::from(rgb[2]);
        }
        let status = if paused {
            "Paused"
        } else if game.dialogue.is_some() {
            "Up/Down: choice | Left/Right: page | Enter/Z: select"
        } else if game.transition.is_some() {
            "Travelling…"
        } else if game.pending_transition.is_some() {
            "This entrance is not available in this build"
        } else {
            "WASD/arrows: walk | F: fire | T: target | Tab: switch | Space: pause | R: reset"
        };
        let line_status = match game.selected_fire_probe(&rom)?.map(|probe| probe.outcome) {
            Some(shadowrealm::combat::LineOutcome::Blocked) => " [path blocked]",
            Some(shadowrealm::combat::LineOutcome::OutsideModel) => " [path unknown]",
            _ => "",
        };
        let target_status = if game.targeting.enabled {
            game.selected_target()
                .map(|r| format!(" | Target: {}{line_status}", r.name()))
                .unwrap_or_else(|| " | No visible target".to_string())
        } else {
            String::new()
        };
        window.set_title(&format!(
            "Shadowrealm — {} — {status}{target_status}{}",
            game.party[game.selected].name(),
            if game.combat.status.is_empty() {
                String::new()
            } else {
                format!(" | {}", game.combat.status)
            }
        ));
        window.update_with_buffer(&buffer, 320, 224)?;
    }
    Ok(())
}
