use crate::tools::Tools;
use serde_json::Value;
use std::{collections::BTreeMap, process::Command};

pub struct Quality {
    pub res: u32,
    pub fps: u32,
}

impl Quality {
    pub fn label(&self) -> String {
        let mut s = format!("{}p", self.res);
        if self.fps > 30 {
            s += &self.fps.to_string();
        }
        s
    }
}

pub struct Probe {
    pub title: String,
    pub qualities: Vec<Quality>,
}

pub fn probe(tools: &Tools, url: &str) -> anyhow::Result<Probe> {
    let out = Command::new(&tools.ytdlp)
        .args(["-J", "--no-playlist", "--no-warnings", url])
        .output()?;
    anyhow::ensure!(
        out.status.success(),
        "could not read video info: {}",
        String::from_utf8_lossy(&out.stderr).trim()
    );

    let json: Value = serde_json::from_slice(&out.stdout)?;
    let title = json["title"].as_str().unwrap_or("video").to_string();

    let mut best: BTreeMap<u32, u32> = BTreeMap::new();
    for f in json["formats"].as_array().into_iter().flatten() {
        if f["vcodec"].as_str().unwrap_or("none") == "none" {
            continue;
        }
        let Some(height) = f["height"].as_u64() else {
            continue;
        };
        let width = f["width"].as_u64().unwrap_or(height);
        let res = width.min(height) as u32;
        let fps = f["fps"].as_f64().unwrap_or(30.0).round() as u32;
        let entry = best.entry(res).or_insert(0);
        *entry = (*entry).max(fps);
    }

    anyhow::ensure!(!best.is_empty(), "no video formats found");

    let mut qualities: Vec<Quality> = best
        .into_iter()
        .map(|(res, fps)| Quality { res, fps })
        .collect();
    qualities.reverse();

    Ok(Probe { title, qualities })
}
