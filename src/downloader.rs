use crate::{tools::Tools, ui::Msg};
use std::{
    fs,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    thread,
};

pub fn spawn(
    tools: &Tools,
    url: String,
    dir: PathBuf,
    res: u32,
    cancel: Arc<AtomicBool>,
) -> Receiver<Msg> {
    let (tx, rx) = mpsc::channel();
    let ytdlp = tools.ytdlp.clone();
    let ffmpeg = tools.ffmpeg.clone();

    thread::spawn(move || {
        let result = run(&ytdlp, &ffmpeg, &url, &dir, res, &tx, &cancel);
        let _ = tx.send(Msg::Finished(result.map_err(|e| e.to_string())));
    });

    rx
}

fn run(
    ytdlp: &Path,
    ffmpeg: &Path,
    url: &str,
    dir: &Path,
    res: u32,
    tx: &Sender<Msg>,
    cancel: &AtomicBool,
) -> anyhow::Result<()> {
    fs::create_dir_all(dir)?;

    let mut child = Command::new(ytdlp)
        .args([
            "--no-playlist",
            "--no-warnings",
            "--no-color",
            "--newline",
            "--quiet",
            "--progress",
            "--progress-template",
            "download:%(progress._percent_str)s",
            "--merge-output-format",
            "mp4",
            "-S",
        ])
        .arg(format!("res:{res},fps,vcodec:h264,acodec:aac"))
        .arg("--ffmpeg-location")
        .arg(ffmpeg)
        .arg("-o")
        .arg(dir.join("%(title)s [%(id)s].%(ext)s"))
        .arg(url)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("could not read yt-dlp output"))?;

    for line in BufReader::new(stdout).lines() {
        if cancel.load(Ordering::Relaxed) {
            let _ = child.kill();
            let _ = child.wait();
            anyhow::bail!("cancelled");
        }
        let line = line?;
        if let Ok(p) = line.trim().trim_end_matches('%').trim().parse::<f64>() {
            let _ = tx.send(Msg::Percent(p));
        }
    }

    let mut err = String::new();
    if let Some(mut stderr) = child.stderr.take() {
        let _ = stderr.read_to_string(&mut err);
    }

    let status = child.wait()?;
    anyhow::ensure!(status.success(), "download failed: {}", err.trim());
    Ok(())
}
