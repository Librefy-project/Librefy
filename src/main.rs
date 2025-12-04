use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box, Orientation, Button, Label, 
    ScrolledWindow, CssProvider, ProgressBar, Window,
    FileDialog, Entry, ListBox, ListBoxRow
};
use std::rc::Rc;
use std::cell::RefCell;
use std::path::PathBuf;

mod player;
mod library;
use player::MusicPlayer;
use library::{MusicLibrary, Song};

const APP_ID: &str = "org.librefy.Librefy";

struct AppState {
    is_dark_theme: bool,
    player: Option<MusicPlayer>,
    library: MusicLibrary,
    current_song: Option<Song>,
}

impl AppState {
    fn new() -> Self {
        Self {
            is_dark_theme: false,
            player: None,
            library: MusicLibrary::new(),
            current_song: None,
        }
    }
}

fn main() -> glib::ExitCode {
    env_logger::init();
    
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        let state = Rc::new(RefCell::new(AppState::new()));
        build_ui(app, state);
    });
    
    app.run()
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_string(include_str!("../data/themes/librefy.css"));
    
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn build_ui(app: &Application, state: Rc<RefCell<AppState>>) {
    load_css();
    
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Librefy")
        .default_width(1200)
        .default_height(800)
        .build();

    window.add_css_class("light-theme");

    // Inicializa o player
    {
        let mut app_state = state.borrow_mut();
        match MusicPlayer::new() {
            Ok(player) => {
                app_state.player = Some(player);
                println!("Player initialized successfully");
            }
            Err(e) => {
                eprintln!("Failed to initialize player: {}", e);
            }
        }
    }

    let main_vertical = Box::new(Orientation::Vertical, 0);
    main_vertical.add_css_class("light-theme");
    
    let content_horizontal = Box::new(Orientation::Horizontal, 0);
    content_horizontal.add_css_class("light-theme");
    
    // Cria a área de conteúdo primeiro
    let (content_scroll, content_box) = create_content_area(&state);
    
    // Cria a sidebar passando o content_box
    let sidebar = create_sidebar(&state, &window, &main_vertical, &content_horizontal, &content_box, &content_scroll);
    sidebar.set_size_request(250, -1);
    sidebar.add_css_class("light-theme");
    content_horizontal.append(&sidebar);
    
    content_scroll.add_css_class("light-theme");
    content_horizontal.append(&content_scroll);
    
    let player_controls = create_player_controls(&state);
    player_controls.set_size_request(-1, 100);
    player_controls.add_css_class("light-theme");
    
    main_vertical.append(&content_horizontal);
    main_vertical.append(&player_controls);
    
    content_horizontal.set_vexpand(true);
    content_scroll.set_hexpand(true);
    content_scroll.set_vexpand(true);

    window.set_child(Some(&main_vertical));
    window.present();
}

fn update_song_list(content: &Box, state: &Rc<RefCell<AppState>>) {
    // Limpa o conteúdo atual
    while let Some(child) = content.first_child() {
        content.remove(&child);
    }

    let app_state = state.borrow();
    
    if app_state.library.songs.is_empty() {
        // Mostra mensagem de boas-vindas se não há músicas
        let welcome_label = Label::new(Some("Welcome to Librefy! 🎵"));
        welcome_label.add_css_class("title-1");
        content.append(&welcome_label);
        
        let subtitle = Label::new(Some("Click \"Add Song\" to add music or \"Scan Music Folder\" to import your library"));
        subtitle.add_css_class("title-4");
        content.append(&subtitle);
    } else {
        // Mostra a lista de músicas
        let songs_label = Label::new(Some("Your Music Library"));
        songs_label.add_css_class("song-list-header");
        content.append(&songs_label);

        let song_list = ListBox::new();
        song_list.add_css_class("song-list");
        
        for song in &app_state.library.songs {
            let row = create_song_row(song, state);
            song_list.append(&row);
        }
        
        content.append(&song_list);
    }
}

fn show_add_song_dialog(parent: &ApplicationWindow, state: &Rc<RefCell<AppState>>, content: &Box) {
    let dialog = Window::builder()
        .title("Add New Song")
        .transient_for(parent)
        .modal(true)
        .default_width(400)
        .default_height(500)
        .build();

    let content_area = Box::new(Orientation::Vertical, 10);
    content_area.set_margin_top(15);
    content_area.set_margin_bottom(15);
    content_area.set_margin_start(15);
    content_area.set_margin_end(15);

    // File chooser for audio file
    let audio_file_btn = Button::with_label("Select Audio File");
    let audio_file_label = Label::new(Some("No file selected"));
    
    let audio_file_state = Rc::new(RefCell::new(None::<PathBuf>));
    
    // Clone dos valores antes de mover para os closures
    let dialog_clone1 = dialog.clone();
    let audio_file_label_clone = audio_file_label.clone();
    let audio_file_state_clone1 = Rc::clone(&audio_file_state);
    
    audio_file_btn.connect_clicked(move |_| {
        let chooser = FileDialog::builder()
            .title("Select Audio File")
            .modal(true)
            .build();
        
        let label_clone = audio_file_label_clone.clone();
        let state_clone = Rc::clone(&audio_file_state_clone1);
        let dialog_clone = dialog_clone1.clone();
        
        chooser.open(Some(&dialog_clone), gio::Cancellable::NONE, move |result| {
            match result {
                Ok(file) => {
                    if let Some(path) = file.path() {
                        *state_clone.borrow_mut() = Some(path.clone());
                        label_clone.set_text(&format!("Selected: {}", path.display()));
                    }
                }
                Err(e) => {
                    eprintln!("Error selecting file: {}", e);
                }
            }
        });
    });

    // File chooser for cover image
    let cover_file_btn = Button::with_label("Select Cover Image (Optional)");
    let cover_file_label = Label::new(Some("No cover selected"));
    
    let cover_file_state = Rc::new(RefCell::new(None::<PathBuf>));
    
    // Clone dos valores antes de mover para os closures
    let dialog_clone2 = dialog.clone();
    let cover_file_label_clone = cover_file_label.clone();
    let cover_file_state_clone1 = Rc::clone(&cover_file_state);
    
    cover_file_btn.connect_clicked(move |_| {
        let chooser = FileDialog::builder()
            .title("Select Cover Image")
            .modal(true)
            .build();
        
        let label_clone = cover_file_label_clone.clone();
        let state_clone = Rc::clone(&cover_file_state_clone1);
        let dialog_clone = dialog_clone2.clone();
        
        chooser.open(Some(&dialog_clone), gio::Cancellable::NONE, move |result| {
            match result {
                Ok(file) => {
                    if let Some(path) = file.path() {
                        *state_clone.borrow_mut() = Some(path.clone());
                        label_clone.set_text(&format!("Selected: {}", path.display()));
                    }
                }
                Err(e) => {
                    eprintln!("Error selecting file: {}", e);
                }
            }
        });
    });

    // Entry fields
    let title_entry = Entry::new();
    title_entry.set_placeholder_text(Some("Song Title"));
    
    let artist_entry = Entry::new();
    artist_entry.set_placeholder_text(Some("Artist"));
    
    let album_entry = Entry::new();
    album_entry.set_placeholder_text(Some("Album"));

    // Add button
    let add_btn = Button::with_label("Add Song");
    add_btn.add_css_class("suggested-action");

    let state_clone = Rc::clone(state);
    let dialog_clone3 = dialog.clone();
    let content_clone = content.clone();
    
    // Clone dos entries e states antes de mover para o closure
    let title_entry_clone = title_entry.clone();
    let artist_entry_clone = artist_entry.clone();
    let album_entry_clone = album_entry.clone();
    let audio_file_state_clone2 = Rc::clone(&audio_file_state);
    let cover_file_state_clone2 = Rc::clone(&cover_file_state);
    
    add_btn.connect_clicked(move |_| {
        // Coletar todos os dados ANTES de fazer qualquer borrow
        let audio_path = audio_file_state_clone2.borrow().clone();
        let cover_path = cover_file_state_clone2.borrow().clone();
        let title = title_entry_clone.text().to_string();
        let artist = artist_entry_clone.text().to_string();
        let album = album_entry_clone.text().to_string();

        // Fechar o diálogo primeiro
        dialog_clone3.close();

        // Processar a adição da música de forma assíncrona
        if let Some(audio_path) = audio_path {
            if !title.is_empty() && !artist.is_empty() {
                let song = Song::new(
                    audio_path,
                    title,
                    artist,
                    if album.is_empty() { "Unknown Album".to_string() } else { album },
                    cover_path,
                );

                // Adicionar a música ao state
                {
                    let mut app_state = state_clone.borrow_mut();
                    app_state.library.add_song(song);
                    println!("Song added to library!");
                }
                
                // Atualizar a UI de forma segura usando timeout - clonando aqui dentro
                let content_for_update = content_clone.clone();
                let state_for_update = Rc::clone(&state_clone);
                let _ = glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                    update_song_list(&content_for_update, &state_for_update);
                    glib::ControlFlow::Break
                });
            } else {
                println!("Please fill title and artist");
            }
        } else {
            println!("Please select an audio file");
        }
    });

    // Cancel button
    let cancel_btn = Button::with_label("Cancel");
    let dialog_clone4 = dialog.clone();
    cancel_btn.connect_clicked(move |_| {
        dialog_clone4.close();
    });

    // Layout
    content_area.append(&Label::new(Some("Audio File:")));
    content_area.append(&audio_file_btn);
    content_area.append(&audio_file_label);
    
    content_area.append(&Label::new(Some("Cover Image:")));
    content_area.append(&cover_file_btn);
    content_area.append(&cover_file_label);
    
    content_area.append(&Label::new(Some("Title:")));
    content_area.append(&title_entry);
    
    content_area.append(&Label::new(Some("Artist:")));
    content_area.append(&artist_entry);
    
    content_area.append(&Label::new(Some("Album:")));
    content_area.append(&album_entry);

    let button_box = Box::new(Orientation::Horizontal, 10);
    button_box.set_halign(gtk4::Align::End);
    button_box.append(&cancel_btn);
    button_box.append(&add_btn);
    
    content_area.append(&button_box);

    dialog.set_child(Some(&content_area));
    dialog.present();
}

fn create_song_row(song: &Song, state: &Rc<RefCell<AppState>>) -> ListBoxRow {
    let row = ListBoxRow::new();
    let row_box = Box::new(Orientation::Horizontal, 10);
    row_box.set_margin_top(5);
    row_box.set_margin_bottom(5);
    row_box.set_margin_start(10);
    row_box.set_margin_end(10);

    // Album art or placeholder
    let album_art = Label::new(Some("🎵"));
    album_art.add_css_class("song-album-art");
    album_art.set_size_request(40, 40);

    // Song info
    let song_info = Box::new(Orientation::Vertical, 2);
    song_info.set_hexpand(true);
    
    let title_label = Label::new(Some(&song.title));
    title_label.add_css_class("song-title");
    title_label.set_halign(gtk4::Align::Start);
    
    let artist_label = Label::new(Some(&format!("{} • {}", song.artist, song.album)));
    artist_label.add_css_class("song-artist");
    artist_label.set_halign(gtk4::Align::Start);

    song_info.append(&title_label);
    song_info.append(&artist_label);

    // Play button
    let play_btn = Button::with_label("▶");
    play_btn.add_css_class("song-play-btn");

    let state_clone = Rc::clone(state);
    let song_clone = song.clone();
    play_btn.connect_clicked(move |_| {
        let mut app_state = state_clone.borrow_mut();
        if let Some(ref mut player) = app_state.player {
            if let Some(path_str) = song_clone.file_path.to_str() {
                match player.load_file(path_str) {
                    Ok(_) => {
                        match player.play() {
                            Ok(_) => {
                                app_state.current_song = Some(song_clone.clone());
                                println!("Now playing: {}", song_clone.get_display_name());
                            }
                            Err(e) => {
                                eprintln!("Failed to play: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to load: {}", e);
                    }
                }
            }
        }
    });

    row_box.append(&album_art);
    row_box.append(&song_info);
    row_box.append(&play_btn);
    
    row.set_child(Some(&row_box));
    row
}

fn create_sidebar(
    state: &Rc<RefCell<AppState>>,
    window: &ApplicationWindow,
    main_vertical: &Box,
    content_horizontal: &Box,
    content: &Box,
    content_scroll: &ScrolledWindow
) -> Box {
    let sidebar = Box::new(Orientation::Vertical, 10);
    sidebar.add_css_class("sidebar");
    sidebar.set_margin_top(10);
    sidebar.set_margin_bottom(10);
    sidebar.set_margin_start(10);
    sidebar.set_margin_end(10);
    
    // Library section
    let library_label = Label::new(Some("📁 Library"));
    library_label.set_halign(gtk4::Align::Start);
    sidebar.append(&library_label);
    
    // Botão "All Songs" para mostrar todas as músicas
    let all_songs_btn = Button::with_label("🎵 All Songs");
    all_songs_btn.set_halign(gtk4::Align::Start);
    all_songs_btn.set_hexpand(false);
    
    let state_clone = Rc::clone(state);
    let content_clone = content.clone();
    all_songs_btn.connect_clicked(move |_| {
        update_song_list(&content_clone, &state_clone);
    });
    
    sidebar.append(&all_songs_btn);
    
    let other_items = vec!["🎤 Artists", "💿 Albums", "🎼 Genres"];
    for item in other_items {
        let button = Button::with_label(item);
        button.set_halign(gtk4::Align::Start);
        button.set_hexpand(false);
        sidebar.append(&button);
    }
    
    let separator = gtk4::Separator::new(Orientation::Horizontal);
    sidebar.append(&separator);
    
    // Add Song button
    let add_song_btn = Button::with_label("➕ Add Song");
    add_song_btn.set_halign(gtk4::Align::Start);
    
    let state_clone = Rc::clone(state);
    let window_clone = window.clone();
    let content_clone = content.clone();
    add_song_btn.connect_clicked(move |_| {
        show_add_song_dialog(&window_clone, &state_clone, &content_clone);
    });
    
    sidebar.append(&add_song_btn);
    
    // Playlists section
    let playlists_label = Label::new(Some("📋 Playlists"));
    playlists_label.set_halign(gtk4::Align::Start);
    sidebar.append(&playlists_label);
    
    let new_playlist_btn = Button::with_label("➕ New Playlist");
    new_playlist_btn.set_halign(gtk4::Align::Start);
    new_playlist_btn.set_hexpand(false);
    sidebar.append(&new_playlist_btn);
    
    // Scan music button
    let scan_music_btn = Button::with_label("🔍 Scan Music Folder");
    scan_music_btn.set_halign(gtk4::Align::Start);
    
    let state_clone = Rc::clone(state);
    let content_clone = content.clone();
    scan_music_btn.connect_clicked(move |_| {
        let mut app_state = state_clone.borrow_mut();
        
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let music_dir = format!("{}/Music", home_dir);
        
        match app_state.library.scan_directory(&music_dir) {
            Ok(_) => {
                println!("Successfully scanned music library. Found {} songs.", app_state.library.songs.len());
                for song in app_state.library.get_all_songs() {
                    println!("Found: {}", song.get_display_name());
                }
                // Atualiza a lista de músicas de forma segura - clonando aqui dentro
                let content_for_update = content_clone.clone();
                let state_for_update = Rc::clone(&state_clone);
                let _ = glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                    update_song_list(&content_for_update, &state_for_update);
                    glib::ControlFlow::Break
                });
            }
            Err(e) => {
                eprintln!("Failed to scan music directory: {}", e);
                println!("Please make sure ~/Music directory exists and contains music files.");
            }
        }
    });
    
    sidebar.append(&scan_music_btn);
    
    // Theme toggle
    let theme_toggle = Button::with_label("🌙 Dark Mode");
    theme_toggle.set_halign(gtk4::Align::Start);
    
    let state_clone = Rc::clone(state);
    let window_clone = window.clone();
    let main_vertical_clone = main_vertical.clone();
    let content_horizontal_clone = content_horizontal.clone();
    let sidebar_clone = sidebar.clone();
    let content_scroll_clone = content_scroll.clone();
    
    theme_toggle.connect_clicked(move |btn| {
        let mut app_state = state_clone.borrow_mut();
        app_state.is_dark_theme = !app_state.is_dark_theme;
        
        if app_state.is_dark_theme {
            btn.set_label("☀️ Light Mode");
            
            window_clone.remove_css_class("light-theme");
            window_clone.add_css_class("dark-theme");
            
            main_vertical_clone.remove_css_class("light-theme");
            main_vertical_clone.add_css_class("dark-theme");
            
            content_horizontal_clone.remove_css_class("light-theme");
            content_horizontal_clone.add_css_class("dark-theme");
            
            sidebar_clone.remove_css_class("light-theme");
            sidebar_clone.add_css_class("dark-theme");
            
            content_scroll_clone.remove_css_class("light-theme");
            content_scroll_clone.add_css_class("dark-theme");
            
            println!("🌙 Dark theme activated");
        } else {
            btn.set_label("🌙 Dark Mode");
            
            window_clone.remove_css_class("dark-theme");
            window_clone.add_css_class("light-theme");
            
            main_vertical_clone.remove_css_class("dark-theme");
            main_vertical_clone.add_css_class("light-theme");
            
            content_horizontal_clone.remove_css_class("dark-theme");
            content_horizontal_clone.add_css_class("light-theme");
            
            sidebar_clone.remove_css_class("dark-theme");
            sidebar_clone.add_css_class("light-theme");
            
            content_scroll_clone.remove_css_class("dark-theme");
            content_scroll_clone.add_css_class("light-theme");
            
            println!("☀️ Light theme activated");
        }
    });
    
    sidebar.append(&theme_toggle);
    
    sidebar
}

fn create_content_area(state: &Rc<RefCell<AppState>>) -> (ScrolledWindow, Box) {
    let scroll = ScrolledWindow::new();
    let content = Box::new(Orientation::Vertical, 20);
    content.set_margin_top(20);
    content.set_margin_bottom(20);
    content.set_margin_start(20);
    content.set_margin_end(20);
    
    // Adiciona a classe do tema ao content box também
    content.add_css_class("light-theme");
    
    // Inicializa com a lista de músicas (ou mensagem de boas-vindas)
    update_song_list(&content, state);
    
    scroll.set_child(Some(&content));
    (scroll, content)
}

fn create_player_controls(state: &Rc<RefCell<AppState>>) -> Box {
    let controls = Box::new(Orientation::Horizontal, 15);
    controls.add_css_class("player-controls");
    controls.set_margin_top(15);
    controls.set_margin_bottom(15);
    controls.set_margin_start(15);
    controls.set_margin_end(15);
    controls.set_valign(gtk4::Align::Center);
    
    // Album art
    let album_art = Label::new(Some("🎵"));
    album_art.add_css_class("album-art");
    controls.append(&album_art);
    
    // Song info
    let song_info = Box::new(Orientation::Vertical, 5);
    let song_title = Label::new(Some("No song playing"));
    song_title.add_css_class("song-title");
    song_title.set_halign(gtk4::Align::Start);
    let song_artist = Label::new(Some("Select a song to start"));
    song_artist.add_css_class("song-artist");
    song_artist.set_halign(gtk4::Align::Start);
    
    song_info.append(&song_title);
    song_info.append(&song_artist);
    controls.append(&song_info);
    
    // Control buttons
    let control_buttons = Box::new(Orientation::Horizontal, 10);
    control_buttons.set_valign(gtk4::Align::Center);
    
    // Previous button
    let prev_btn = Button::with_label("⏮");
    let state_prev = Rc::clone(state);
    prev_btn.connect_clicked(move |_| {
        let _app_state = state_prev.borrow();
        println!("Previous track");
    });
    
    // Play/Pause button
    let play_btn = Button::with_label("⏯");
    play_btn.add_css_class("suggested-action");
    let state_play = Rc::clone(state);
    play_btn.connect_clicked(move |btn| {
        let mut app_state = state_play.borrow_mut();
        if let Some(ref mut player) = app_state.player {
            if player.is_playing() {
                player.pause();
                btn.set_label("⏯");
                println!("Paused");
            } else {
                match player.play() {
                    Ok(_) => {
                        btn.set_label("⏸");
                        println!("Playing");
                    }
                    Err(e) => {
                        eprintln!("Failed to play: {}", e);
                    }
                }
            }
        }
    });
    
    // Next button
    let next_btn = Button::with_label("⏭");
    let state_next = Rc::clone(state);
    next_btn.connect_clicked(move |_| {
        let _app_state = state_next.borrow();
        println!("Next track");
    });
    
    control_buttons.append(&prev_btn);
    control_buttons.append(&play_btn);
    control_buttons.append(&next_btn);
    controls.append(&control_buttons);
    
    // Progress area
    let progress_area = Box::new(Orientation::Vertical, 5);
    progress_area.set_hexpand(true);
    progress_area.set_valign(gtk4::Align::Center);
    
    // Progress bar
    let progress_bar = ProgressBar::new();
    progress_bar.set_hexpand(true);
    
    // Time labels
    let time_labels = Box::new(Orientation::Horizontal, 0);
    time_labels.set_hexpand(true);
    
    let current_time = Label::new(Some("0:00"));
    current_time.add_css_class("time-label");
    current_time.set_halign(gtk4::Align::Start);
    
    let total_time = Label::new(Some("0:00"));
    total_time.add_css_class("time-label");
    total_time.set_halign(gtk4::Align::End);
    
    time_labels.append(&current_time);
    time_labels.append(&total_time);
    
    progress_area.append(&progress_bar);
    progress_area.append(&time_labels);
    controls.append(&progress_area);
    
    // Volume control
    let volume_btn = Button::with_label("🔊");
    let state_volume = Rc::clone(state);
    volume_btn.connect_clicked(move |_| {
        let app_state = state_volume.borrow();
        if let Some(ref player) = app_state.player {
            player.set_volume(0.5);
            println!("Volume set to 50%");
        }
    });
    
    controls.append(&volume_btn);
    
    controls
}
