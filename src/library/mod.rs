use std::path::{Path, PathBuf};
use std::collections::HashMap;
use anyhow::Result;
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    pub file_path: PathBuf,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub cover_path: Option<PathBuf>, // Nova: caminho para a capa
}

#[derive(Debug, Clone)]
pub struct MusicLibrary {
    pub songs: Vec<Song>,
    pub artists: HashMap<String, Vec<Song>>,
    pub albums: HashMap<String, Vec<Song>>,
}

impl MusicLibrary {
    pub fn new() -> Self {
        Self {
            songs: Vec::new(),
            artists: HashMap::new(),
            albums: HashMap::new(),
        }
    }

    pub fn add_song(&mut self, song: Song) {
        self.songs.push(song.clone());
        self.organize_library();
    }

    pub fn scan_directory(&mut self, path: &str) -> Result<()> {
        let path = Path::new(path);
        if !path.exists() {
            return Err(anyhow::anyhow!("Directory does not exist: {}", path.display()));
        }

        println!("Scanning directory: {}", path.display());
        self.scan_directory_recursive(path)?;
        println!("Found {} songs", self.songs.len());
        
        self.organize_library();
        Ok(())
    }

    fn scan_directory_recursive(&mut self, path: &Path) -> Result<()> {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    
                    if path.is_dir() {
                        self.scan_directory_recursive(&path)?;
                    } else if self.is_audio_file(&path) {
                        match self.create_song_from_file(&path) {
                            Ok(song) => {
                                self.songs.push(song);
                            }
                            Err(e) => {
                                eprintln!("Error reading file {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn is_audio_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            matches!(ext.as_str(), "mp3" | "flac" | "wav" | "ogg" | "m4a" | "aac")
        } else {
            false
        }
    }

    fn create_song_from_file(&self, path: &Path) -> Result<Song> {
        let file_name = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let (artist, title) = self.parse_file_name(&file_name);

        Ok(Song {
            file_path: path.to_path_buf(),
            title,
            artist,
            album: "Unknown Album".to_string(),
            cover_path: None,
        })
    }

    fn parse_file_name(&self, file_name: &str) -> (String, String) {
        let separators = [" - ", " – ", " — "];
        
        for separator in &separators {
            if let Some(pos) = file_name.find(separator) {
                let artist = file_name[..pos].trim().to_string();
                let title = file_name[pos + separator.len()..].trim().to_string();
                
                if !artist.is_empty() && !title.is_empty() {
                    return (artist, title);
                }
            }
        }
        
        ("Unknown Artist".to_string(), file_name.to_string())
    }

    fn organize_library(&mut self) {
        self.artists.clear();
        self.albums.clear();

        for song in &self.songs {
            self.artists
                .entry(song.artist.clone())
                .or_insert_with(Vec::new)
                .push(song.clone());

            self.albums
                .entry(song.album.clone())
                .or_insert_with(Vec::new)
                .push(song.clone());
        }
    }

    pub fn get_all_songs(&self) -> &[Song] {
        &self.songs
    }

    pub fn search_songs(&self, query: &str) -> Vec<&Song> {
        let query = query.to_lowercase();
        self.songs
            .iter()
            .filter(|song| {
                song.title.to_lowercase().contains(&query) ||
                song.artist.to_lowercase().contains(&query) ||
                song.album.to_lowercase().contains(&query)
            })
            .collect()
    }
}

impl Song {
    pub fn new(file_path: PathBuf, title: String, artist: String, album: String, cover_path: Option<PathBuf>) -> Self {
        Self {
            file_path,
            title,
            artist,
            album,
            cover_path,
        }
    }

    pub fn get_display_name(&self) -> String {
        format!("{} - {}", self.artist, self.title)
    }
    
    pub fn get_file_path(&self) -> &Path {
        &self.file_path
    }

    pub fn get_cover_path(&self) -> Option<&Path> {
        self.cover_path.as_deref()
    }
}
