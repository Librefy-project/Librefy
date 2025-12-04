use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub struct MusicPlayer {
    current_file: Option<String>,
    is_playing: bool,
    child_process: Option<std::process::Child>,
}

impl MusicPlayer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            current_file: None,
            is_playing: false,
            child_process: None,
        })
    }

    pub fn load_file(&mut self, file_path: &str) -> Result<()> {
        let path = Path::new(file_path);
        if path.exists() {
            self.current_file = Some(file_path.to_string());
            println!("Loaded: {}", file_path);
            Ok(())
        } else {
            Err(anyhow::anyhow!("File not found: {}", file_path))
        }
    }

    pub fn play(&mut self) -> Result<()> {
        if let Some(ref file_path) = self.current_file {
            // Tenta usar mpv primeiro, depois vlc, depois ffplay
            let players = ["mpv", "vlc", "ffplay"];
            
            for player in players {
                let result = Command::new(player)
                    .arg("--no-video")
                    .arg("--no-terminal")
                    .arg(file_path)
                    .spawn();
                
                match result {
                    Ok(child) => {
                        self.child_process = Some(child);
                        self.is_playing = true;
                        println!("Playing with {}: {}", player, file_path);
                        return Ok(());
                    }
                    Err(_) => continue,
                }
            }
            
            Err(anyhow::anyhow!("No supported media player found. Please install mpv, vlc, or ffmpeg."))
        } else {
            Err(anyhow::anyhow!("No file loaded"))
        }
    }

    pub fn pause(&mut self) {
        // Para processos externos, pausar é complexo, então vamos apenas parar
        self.stop();
        self.is_playing = false;
    }

    pub fn stop(&mut self) {
        if let Some(ref mut child) = self.child_process {
            let _ = child.kill();
            self.child_process = None;
        }
        self.is_playing = false;
    }

    pub fn set_volume(&self, _volume: f64) {
        // Volume control não suportado nesta versão simples
        println!("Volume control not supported in basic player");
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn current_file(&self) -> Option<&String> {
        self.current_file.as_ref()
    }
}
