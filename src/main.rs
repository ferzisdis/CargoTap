use anyhow::Result;
use log::info;
use winit::event_loop::EventLoop;

mod app;
mod char_utils;
mod code_state;
mod config;
mod demo_code_state;
mod event_handler;
mod input;
mod profiling;
mod progress_helper;
mod progress_storage;
mod renderer;
mod session_history;
mod session_state;
mod text;
mod typing_handler;
mod ui;

mod examples;
use examples::colored_text_demo::ColoredTextDemo;

use app::CargoTapApp;

fn main() -> Result<()> {
    let config = config::Config::load();
    simple_logger::init_with_level(config.get_log_level()).unwrap();

    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "gen-config" {
        match config::Config::save_default("config.toml") {
            Ok(_) => {
                println!("✓ Default configuration saved to config.toml");
                println!("  Edit this file to customize CargoTap settings.");
                return Ok(());
            }
            Err(e) => {
                eprintln!("✗ Failed to save config: {}", e);
                return Err(e);
            }
        }
    }

    if args.len() > 1 && args[1] == "demo" {
        demo_code_state::run_demo();
        return Ok(());
    }

    info!("Starting CargoTap application");
    info!("Tip: Run with 'cargo run demo' for command-line demo");
    info!("Tip: Run with 'cargo run gen-config' to generate config.toml");

    let event_loop = EventLoop::new()?;
    let mut app = CargoTapApp::new(&event_loop)?;

    info!("Starting event loop");
    event_loop.run_app(&mut app)?;
    info!("Finished event loop");
    Ok(())
}
