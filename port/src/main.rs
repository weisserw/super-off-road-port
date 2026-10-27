use anyhow::{Context, Result, bail};
use config::ROM_SHA256;
use romatlas_port::{assets::Rom, input::InputState};
use std::path::PathBuf;
use super_off_road_port::{config, game};

fn main() {
    if let Err(error) = run() {
        eprintln!(
            "{}",
            serde_json::json!({"ok":false,"error":format!("{error:#}")})
        );
        std::process::exit(1);
    }
}
fn run() -> Result<()> {
    let mut debug_stdio = false;
    let mut rom = None;
    let mut assets = None;
    let mut settings = None;
    let mut frames: Option<usize> = None;
    let mut random_history = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--debug-stdio" => debug_stdio = true,
            "--rom" => rom = Some(PathBuf::from(args.next().context("--rom needs a file")?)),
            "--assets" => {
                assets = Some(PathBuf::from(
                    args.next().context("--assets needs a directory")?,
                ))
            }
            "--settings" => {
                settings = Some(PathBuf::from(
                    args.next().context("--settings needs a file")?,
                ))
            }
            "--headless" => {
                frames = Some(
                    args.next()
                        .context("--headless needs a frame count")?
                        .parse()?,
                )
            }
            "--rng-seed" => {
                let hex = args
                    .next()
                    .context("--rng-seed needs ten hexadecimal digits")?;
                anyhow::ensure!(
                    hex.len() == super_off_road_port::runtime::RNG_STATE_BYTES * 2
                        && hex.bytes().all(|byte| byte.is_ascii_hexdigit()),
                    "--rng-seed needs ten hexadecimal digits"
                );
                let mut history = [0; super_off_road_port::runtime::RNG_STATE_BYTES];
                for (index, byte) in history.iter_mut().enumerate() {
                    *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16)?;
                }
                random_history = Some(history);
            }
            "--help" => {
                println!(
                    "Usage: super-off-road --rom FILE [--assets DIR] [--settings FILE] [--debug-stdio] [--headless FRAMES] [--rng-seed HEX]\n--rng-seed is for reproducible headless/debug runs only; normal games use host randomness.\nF1 resume, F2 pause, F3 reset, F4 save state, F5 restore state, F6 settings."
                );
                return Ok(());
            }
            _ => bail!("unknown argument: {arg}"),
        }
    }
    let rom = Rom::open(
        &rom.context("supply your matching ROM with --rom")?,
        ROM_SHA256,
    )?;
    anyhow::ensure!(
        random_history.is_none() || debug_stdio || frames.is_some(),
        "--rng-seed requires --debug-stdio or --headless"
    );
    if debug_stdio {
        anyhow::ensure!(
            frames.is_none() && settings.is_none(),
            "--debug-stdio cannot combine with --headless/--settings"
        );
        let resource_identity = romatlas_port::debug::identity(ROM_SHA256, assets.as_deref())?;
        let history = match random_history {
            Some(history) => history,
            None => super_off_road_port::runtime::RandomState::from_entropy()?.0,
        };
        let seed_identity = history
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let identity = format!("{resource_identity}:rng={seed_identity}");
        return romatlas_port::debug::serve_with_snapshots(
            || {
                anyhow::ensure!(
                    romatlas_port::debug::identity(ROM_SHA256, assets.as_deref())?
                        == resource_identity,
                    "debug assets/build changed; restart session"
                );
                game::Port::load_seeded(&rom, assets.as_deref(), history)
            },
            config::SIZE,
            config::RATE,
            config::settings().bindings.keys().cloned().collect(),
            identity.clone(),
            game::Port::observe,
        );
    }
    let mut port = match random_history {
        Some(history) => game::Port::load_seeded(&rom, assets.as_deref(), history)?,
        None => game::Port::load(&rom, assets.as_deref())?,
    };
    if let Some(frames) = frames {
        let frame =
            romatlas_port::headless(&mut port, frames, |_| InputState::default(), config::SIZE)?;
        println!(
            "{}",
            serde_json::json!({"ok":true,"native":true,"frames":frames,"width":frame.width,"height":frame.height})
        );
        return Ok(());
    }
    #[cfg(feature = "desktop")]
    {
        let path = match settings {
            Some(path) => path,
            None => romatlas_port::settings::default_path("super-off-road")?,
        };
        let battery = path.with_file_name(format!("battery-{}.json", ROM_SHA256));
        romatlas_port::desktop::run_with_assets(
            &mut port,
            "Super Off Road (F6 settings)",
            config::SIZE,
            config::RATE,
            &path,
            config::settings(),
            Some((ROM_SHA256, &battery)),
            assets.as_deref(),
        )?;
        Ok(())
    }
    #[cfg(not(feature = "desktop"))]
    {
        let _ = settings;
        bail!("desktop feature is disabled; use --headless FRAMES or build with --features desktop")
    }
}
