use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
#[cfg(target_os = "windows")]
use sevenz_rust::{Password, SevenZReader};

#[cfg(target_os = "linux")]
use tar::Archive;
#[cfg(target_os = "linux")]
use xz2::read::XzDecoder;
#[cfg(target_os = "linux")]
use std::os::unix::fs::PermissionsExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Clone, Debug)]
pub struct YoutubeDownloader {
    yt_dlp_path: PathBuf,
    ffmpeg_path: PathBuf,
}

impl YoutubeDownloader {
    pub fn new(libs_folder: PathBuf) -> Self {
        Self {
            yt_dlp_path: libs_folder.join("yt-dlp"),
            ffmpeg_path: libs_folder.join("ffmpeg"),
        }
    }

    pub fn check_prerequisites(&self) -> bool {
        self.yt_dlp_path.exists() && self.ffmpeg_path.exists()
    }

    #[cfg(target_os = "linux")]
    pub async fn download_prerequisites(libs_folder: PathBuf) {
        std::fs::create_dir_all(libs_folder.clone()).unwrap();
        let resp = reqwest::get("https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp").await.unwrap().bytes().await.unwrap();
        std::fs::write(libs_folder.join("yt-dlp"), &resp).unwrap();
        let resp = reqwest::get("https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz").await.unwrap().bytes().await.unwrap();
        std::fs::write(libs_folder.join("ffmpeg.tar.xz"), &resp).unwrap();

        let file = File::open(libs_folder.join("ffmpeg.tar.xz")).unwrap();
        let decoder = XzDecoder::new(file);
        let mut archive = Archive::new(decoder);
        for entry in archive.entries().unwrap() {
            let mut entry = entry.unwrap();
            if entry.path().unwrap().to_string_lossy().contains("ffmpeg") {
                let mut output_file = File::create(libs_folder.join("ffmpeg")).unwrap();
                std::io::copy(&mut entry, &mut output_file).unwrap();
            }
        }

        std::fs::set_permissions(libs_folder.join("ffmpeg"), std::fs::Permissions::from_mode(0o744)).unwrap();
        std::fs::set_permissions(libs_folder.join("yt-dlp"), std::fs::Permissions::from_mode(0o744)).unwrap();
        std::fs::remove_file(libs_folder.join("ffmpeg.tar.xz")).unwrap();
    }

    #[cfg(target_os = "windows")]
    pub async fn download_prerequisites(libs_folder: PathBuf) {
        std::fs::create_dir_all(libs_folder.clone()).unwrap();
        let resp = reqwest::get("https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe").await.unwrap().bytes().await.unwrap();
        std::fs::write(libs_folder.join("yt-dlp.exe"), &resp).unwrap();
        let resp = reqwest::get("https://www.gyan.dev/ffmpeg/builds/ffmpeg-git-essentials.7z").await.unwrap().bytes().await.unwrap();
        std::fs::write(libs_folder.join("ffmpeg.7z"), &resp).unwrap();

        let mut archive = SevenZReader::open(libs_folder.join("ffmpeg.7z"), Password::empty()).unwrap();
        archive.for_each_entries( |entry, r| {
            if entry.name().contains("ffmpeg.exe") {
                let mut output_file = File::create(libs_folder.join("ffmpeg.exe")).unwrap();
                std::io::copy(r, &mut output_file).unwrap();
                Ok(true)
            } else {
                Ok(true)
            }
        }).unwrap();

        std::fs::remove_file(libs_folder.join("ffmpeg.7z")).unwrap();
    }

    #[cfg(target_os = "windows")]
    pub fn download(&mut self, url: String, out_path: &Path) -> Child {
        let child = Command::new(self.yt_dlp_path.clone())
            .args(vec![
                url.as_str(),
                "-o",
                "%(title)s",
                "-x",
                "-q",
                "--audio-format",
                "mp3",
                "--no-playlist",
                "--ffmpeg-location",
                self.ffmpeg_path.display().to_string().as_str(),
                "-P",
                out_path.display().to_string().as_str(),
                "--progress",
                "--progress-template",
                "%(progress._percent_str)s"
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .unwrap();

        child
    }

    #[cfg(not(target_os = "windows"))]
    pub fn download(&mut self, url: String, out_path: &Path) -> Child {
        let child = Command::new(self.yt_dlp_path.clone())
            .args(vec![
                url.as_str(),
                "-o",
                "%(title)s",
                "-x",
                "-q",
                "--audio-format",
                "mp3",
                "--no-playlist",
                "--ffmpeg-location",
                self.ffmpeg_path.display().to_string().as_str(),
                "-P",
                out_path.display().to_string().as_str(),
                "--progress",
                "--progress-template",
                "%(progress._percent_str)s"
            ])
            .spawn()
            .unwrap();

        child
    }
}
