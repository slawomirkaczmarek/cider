use std::{fs::File, io::Write, path::PathBuf, process::Command, str::FromStr};

use anyhow::{anyhow, Context, Result};
use reqwest::{blocking::Client, header::{ACCEPT, CONTENT_LENGTH, RANGE, HeaderValue}, StatusCode};
use serde_json::Value;

use crate::settings;

const PART_SIZE: u64 = 8 * 1024 * 1024;

const TMP_FILE_NAME: &str = "update.tmp";

pub struct Artifact {
    pub name: String,
    pub published_at: String,
    pub download_url: String,
}

struct RangeIterator {
    start: u64,
    end: u64,
}

impl Iterator for RangeIterator {
    type Item = HeaderValue;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start > self.end {
            None
        } else {
            let prev_start = self.start;
            self.start = std::cmp::min(self.start + PART_SIZE, self.end + 1);
            Some(HeaderValue::from_str(&format!("bytes={}-{}", prev_start, self.start - 1)).ok()?)
        }
    }
}

fn client() -> Result<Client> {
    Client::builder()
        .user_agent("Cider-App (Rust reqwest Client)")
        .build().context("Unable to build HTTP client")
}

pub fn get_latest() -> Result<(Client, Artifact)> {
    let client = client()?;

    let response = client.get("https://api.github.com/repos/Gcenx/game-porting-toolkit/releases/latest")
        .header(ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()?;

    let json = response.json::<Value>()?;

    let artifact = Artifact {
        name: json["name"].as_str().ok_or(anyhow!("Unable to get artifact name"))?.to_owned(),
        published_at: json["published_at"].as_str().ok_or(anyhow!("Unable to get artifact published date"))?.to_owned(),
        download_url: json["assets"][0]["browser_download_url"].as_str().ok_or(anyhow!("Unable to get artifact download url"))?.to_owned(),
    };

    Ok((client, artifact))
}

pub fn download(client: &Client, url: &String, json: bool) -> Result<PathBuf> {
    let response = client.head(url).send()?;
    let length = response.headers().get(CONTENT_LENGTH).with_context(|| "Unable to get content length")?;
    let length = u64::from_str(length.to_str()?).with_context(|| "Unable to parse content length")?;

    let tmp_file_path = settings::app_support_dir()?.join(TMP_FILE_NAME);
    let mut tmp_file = File::create(&tmp_file_path)?;

    let mut progress: u64 = 0;
    if !json {
        print!("Downloading: [>{}]", " ".repeat(20));
        std::io::stdout().flush()?;
    }

    for range in (RangeIterator { start: 0, end: length - 1 }) {
        let mut response = client.get(url)
            .header(RANGE, range)
            .send()?;

        let status = response.status();
        if status != StatusCode::PARTIAL_CONTENT && status != StatusCode::OK {
            return Err(anyhow!("Unexpected response status: {}", status));
        }

        std::io::copy(&mut response, &mut tmp_file)?;

        progress += PART_SIZE;

        let percentage = (std::cmp::min(progress * 100 / length, 100) / 5) as usize;
        if !json {
            print!("\rDownloading: [{}>{}]", "=".repeat(percentage), " ".repeat(20 - percentage));
            std::io::stdout().flush()?;
        }
    }

    if !json {
        print!("\r{}", " ".repeat(36));
        std::io::stdout().flush()?;
    }

    Ok(tmp_file_path)
}

pub fn extract(archive_file: &PathBuf, dir: &PathBuf, json: bool) -> Result<()> {
    if !json {
        print!("\rExtracting...");
        std::io::stdout().flush()?;
    }

    let mut process = Command::new("tar")
        .arg("-x")
        .arg("-f")
        .arg(archive_file)
        .arg("-C")
        .arg(dir)
        .spawn()?;

    let status = process.wait()?;

    if status.success() {
        if archive_file.exists() {
            std::fs::remove_file(archive_file).with_context(|| format!("Unable to remove archive file `{}`", archive_file.display()))?;
        }
        Ok(())
    } else {
        return Err(anyhow!("Unable to extract archive: {status}"))
    }
}
