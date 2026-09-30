mod downloader;
mod formats;
mod tools;
mod ui;

use clap::Parser;
use std::{
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
};

#[derive(Parser)]
#[command(name = "cf", about = "TikTok/Instagram/YouTube downloader")]
struct Cli {
    url: String,

    #[arg(short, long)]
    output: Option<PathBuf>,

    #[arg(short, long)]
    quality: Option<u32>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let dir = cli
        .output
        .or_else(dirs::download_dir)
        .ok_or_else(|| anyhow::anyhow!("could not find Downloads folder, use --output"))?;

    let tools = tools::ensure_tools()?;

    eprintln!("Reading video info...");
    let probe = formats::probe(&tools, &cli.url)?;

    let res = match cli.quality {
        Some(q) => q,
        None => {
            let default = probe
                .qualities
                .iter()
                .position(|q| q.res <= 1080)
                .unwrap_or(0);
            match ui::select_quality(&probe.title, &probe.qualities, default)? {
                Some(i) => probe.qualities[i].res,
                None => return Ok(()),
            }
        }
    };

    let cancel = Arc::new(AtomicBool::new(false));
    let rx = downloader::spawn(&tools, cli.url.clone(), dir.clone(), res, cancel.clone());

    match ui::show_progress(&probe.title, rx, &cancel)? {
        Ok(()) => println!("Saved to {}", dir.display()),
        Err(e) => anyhow::bail!(e),
    }
    Ok(())
}
