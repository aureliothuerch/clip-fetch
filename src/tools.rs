use flate2::read::GzDecoder;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub struct Tools {
    pub ytdlp: PathBuf,
    pub ffmpeg: PathBuf,
}

fn cache_dir() -> anyhow::Result<PathBuf> {
    let dir = dirs::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("no cache directory found"))?
        .join("cf");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn exe(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

fn ytdlp_asset() -> anyhow::Result<&'static str> {
    Ok(match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", _) => "yt-dlp_macos",
        ("linux", "aarch64") => "yt-dlp_linux_aarch64",
        ("linux", _) => "yt-dlp_linux",
        ("windows", _) => "yt-dlp.exe",
        _ => anyhow::bail!("unsupported platform"),
    })
}

fn ffmpeg_asset() -> anyhow::Result<&'static str> {
    Ok(match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "ffmpeg-darwin-arm64",
        ("macos", _) => "ffmpeg-darwin-x64",
        ("linux", "aarch64") => "ffmpeg-linux-arm64",
        ("linux", _) => "ffmpeg-linux-x64",
        ("windows", _) => "ffmpeg-win32-x64",
        _ => anyhow::bail!("unsupported platform"),
    })
}

#[cfg(unix)]
fn make_executable(path: &Path) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> anyhow::Result<()> {
    Ok(())
}

fn fetch(url: &str, dest: &Path, gzipped: bool) -> anyhow::Result<()> {
    let resp = ureq::get(url).call()?;
    let reader = resp.into_reader();

    let tmp = dest.with_extension("part");
    let mut file = fs::File::create(&tmp)?;
    if gzipped {
        io::copy(&mut GzDecoder::new(reader), &mut file)?;
    } else {
        let mut reader = reader;
        io::copy(&mut reader, &mut file)?;
    }
    drop(file);

    make_executable(&tmp)?;
    fs::rename(&tmp, dest)?;
    Ok(())
}

fn ensure(name: &str, url: String, gzipped: bool) -> anyhow::Result<PathBuf> {
    let path = cache_dir()?.join(exe(name));
    if !path.exists() {
        eprintln!("First run: downloading {name} (one time only)...");
        fetch(&url, &path, gzipped)?;
    }
    Ok(path)
}

pub fn ensure_tools() -> anyhow::Result<Tools> {
    let ytdlp = ensure(
        "yt-dlp",
        format!(
            "https://github.com/yt-dlp/yt-dlp/releases/latest/download/{}",
            ytdlp_asset()?
        ),
        false,
    )?;
    let ffmpeg = ensure(
        "ffmpeg",
        format!(
            "https://github.com/eugeneware/ffmpeg-static/releases/download/b6.1.1/{}.gz",
            ffmpeg_asset()?
        ),
        true,
    )?;
    Ok(Tools { ytdlp, ffmpeg })
}
