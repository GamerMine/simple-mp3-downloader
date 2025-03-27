use std::path::{Path, PathBuf};
use std::process::{Child, Command};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
const _CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Clone, Debug)]
pub struct YoutubeDownloader {
    yt_dlp_path: PathBuf,
    ffmpeg_path: PathBuf,
}

impl YoutubeDownloader {
    /// The libs_folder must contain ffmpeg(.exe) and yt-dlp(.exe)
    #[cfg(target_os = "windows")]
    pub fn new(libs_folder: PathBuf) -> Self {
        let yt_dlp_path = libs_folder.join("yt-dlp.exe").canonicalize().unwrap();

        Command::new(yt_dlp_path.clone())
            .arg("-U")
            .arg("-q")
            .creation_flags(_CREATE_NO_WINDOW)
            .spawn()
            .expect("Unable to update yt-dlp!")
            .wait()
            .expect("TODO: panic message");
        
        Self {
            yt_dlp_path,
            ffmpeg_path: libs_folder.join("ffmpeg.exe").canonicalize().unwrap(),
        }
    }

    #[cfg(not(target_os = "windows"))]
    pub fn new(libs_folder: PathBuf) -> Self {
        let yt_dlp_path = libs_folder.join("yt-dlp").canonicalize().unwrap();

        Command::new(yt_dlp_path.clone())
            .arg("-U")
            .arg("-q")
            .spawn()
            .expect("Unable to update yt-dlp!")
            .wait()
            .expect("TODO: panic message");

        Self {
            yt_dlp_path,
            ffmpeg_path: libs_folder.join("ffmpeg").canonicalize().unwrap(),
        }
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
            .creation_flags(_CREATE_NO_WINDOW)
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
