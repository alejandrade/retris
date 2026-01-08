use crate::volume_manager::VolumeManager;
use kira::{
    AudioManager, AudioManagerSettings, DefaultBackend, Tween,
    sound::{
        PlaybackState,
        static_sound::{StaticSoundData, StaticSoundHandle},
    },
};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub enum LoadingState {
    NotStarted,
    Loading { current: usize, total: usize },
    Complete,
    Failed(String),
}

pub struct MusicManager {
    audio_manager: AudioManager<DefaultBackend>,
    playlist: Arc<Mutex<Vec<(String, StaticSoundData)>>>,
    current_index: usize,
    current_handle: Option<StaticSoundHandle>,
    loading_state: Arc<Mutex<LoadingState>>,
    muted: bool,
    volume_manager: VolumeManager,
}

impl MusicManager {
    /// Create a new MusicManager without loading music yet
    pub fn new(volume_manager: VolumeManager) -> Result<Self, Box<dyn std::error::Error>> {
        let mut audio_manager =
            AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;

        // Set initial volume from volume manager
        let initial_volume = volume_manager.music_volume();
        let db = Self::amplitude_to_db(initial_volume);
        let _ = audio_manager.main_track().set_volume(db, Tween::default());

        println!(
            "MusicManager initialized with volume {} ({:.1} dB)",
            initial_volume, db
        );

        Ok(Self {
            audio_manager,
            playlist: Arc::new(Mutex::new(Vec::new())),
            current_index: 0,
            current_handle: None,
            loading_state: Arc::new(Mutex::new(LoadingState::NotStarted)),
            muted: false,
            volume_manager,
        })
    }

    /// Convert linear amplitude (0.0-1.0) to decibels with better perceptual curve
    fn amplitude_to_db(amplitude: f32) -> f32 {
        if amplitude <= 0.0 {
            -60.0 // Essentially silent
        } else {
            // Use exponential curve for better perceived volume control
            // Square the amplitude to make volume drop more gradually
            let curved = amplitude * amplitude;
            20.0 * curved.log10()
        }
    }

    /// Mute the music (won't play)
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
        if muted {
            // Stop current playback if any
            self.stop_current_song();
        }
    }

    /// Check if music is muted
    pub fn is_muted(&self) -> bool {
        self.muted
    }

    /// Update volume from volume manager (call this periodically or when volume changes)
    pub fn update_volume(&mut self) {
        let volume = self.volume_manager.music_volume();
        let db = Self::amplitude_to_db(volume);
        let _ = self.audio_manager.main_track().set_volume(
            db,
            Tween {
                duration: std::time::Duration::from_millis(100),
                ..Default::default()
            },
        );
    }
    
    /// Play a test sound (plays the first loaded song briefly)
    pub fn test_sound(&mut self) {
        // ALWAYS stop current song first
        self.stop_current_song();
        
        let playlist = self.playlist.lock().unwrap();
        if playlist.is_empty() {
            return;
        }
        
        // Play first song briefly - it will naturally stop after a couple seconds
        // when the next UI interaction happens (slider movement, etc)
        let (_, sound_data) = &playlist[0];
        if let Ok(handle) = self.audio_manager.play(sound_data.clone()) {
            self.current_handle = Some(handle);
        }
    }
    
    /// Stop any test sound that's currently playing
    pub fn stop_test_sound(&mut self) {
        self.stop_current_song();
    }

    /// Start loading music in the background
    /// This is non-blocking and will load music on a separate thread (native) or task (WASM)
    pub fn start_loading_background(&mut self) {
        let playlist = Arc::clone(&self.playlist);
        let loading_state = Arc::clone(&self.loading_state);

        // Mark as loading
        *loading_state.lock().unwrap() = LoadingState::Loading {
            current: 0,
            total: 18,
        };

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen_futures::spawn_local;
            spawn_local(async move {
                Self::load_music_async(playlist, loading_state).await;
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            std::thread::spawn(move || {
                Self::load_music_sync(playlist, loading_state);
            });
        }
    }

    /// Synchronous music loading (for native thread)
    #[cfg(not(target_arch = "wasm32"))]
    fn load_music_sync(
        playlist: Arc<Mutex<Vec<(String, StaticSoundData)>>>,
        loading_state: Arc<Mutex<LoadingState>>,
    ) {
        let song_files = Self::get_song_list();

        for (index, song_file) in song_files.iter().enumerate() {
            // Update loading state
            *loading_state.lock().unwrap() = LoadingState::Loading {
                current: index,
                total: song_files.len(),
            };

            let mut path = PathBuf::from("assets");
            path.push(song_file);

            match StaticSoundData::from_file(&path) {
                Ok(sound_data) => {
                    let name = Self::extract_song_name(song_file);
                    playlist.lock().unwrap().push((name, sound_data));
                    println!("Loaded: {}", song_file);
                }
                Err(e) => {
                    eprintln!("Failed to load {}: {}", song_file, e);
                    *loading_state.lock().unwrap() =
                        LoadingState::Failed(format!("Failed to load {}", song_file));
                    return;
                }
            }
        }

        let count = playlist.lock().unwrap().len();
        if count == 0 {
            *loading_state.lock().unwrap() =
                LoadingState::Failed("No music files loaded".to_string());
        } else {
            *loading_state.lock().unwrap() = LoadingState::Complete;
            println!("Music loading complete: {} songs", count);
        }
    }

    /// Async music loading (for WASM)
    #[cfg(target_arch = "wasm32")]
    async fn load_music_async(
        playlist: Arc<Mutex<Vec<(String, StaticSoundData)>>>,
        loading_state: Arc<Mutex<LoadingState>>,
    ) {
        let song_files = Self::get_song_list();

        for (index, song_file) in song_files.iter().enumerate() {
            // Update loading state
            *loading_state.lock().unwrap() = LoadingState::Loading {
                current: index,
                total: song_files.len(),
            };

            let mut path = PathBuf::from("assets");
            path.push(song_file);

            match StaticSoundData::from_file(&path) {
                Ok(sound_data) => {
                    let name = Self::extract_song_name(song_file);
                    playlist.lock().unwrap().push((name, sound_data));
                    println!("Loaded: {}", song_file);
                }
                Err(e) => {
                    eprintln!("Failed to load {}: {}", song_file, e);
                    *loading_state.lock().unwrap() =
                        LoadingState::Failed(format!("Failed to load {}", song_file));
                    return;
                }
            }

            // Yield to prevent blocking the main thread
            wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(
                &wasm_bindgen::JsValue::NULL,
            ))
            .await
            .ok();
        }

        let count = playlist.lock().unwrap().len();
        if count == 0 {
            *loading_state.lock().unwrap() =
                LoadingState::Failed("No music files loaded".to_string());
        } else {
            *loading_state.lock().unwrap() = LoadingState::Complete;
            println!("Music loading complete: {} songs", count);
        }
    }

    fn get_song_list() -> Vec<&'static str> {
        vec![
            "01. Slay The Evil.ogg",
            "02. Perilous Dungeon.ogg",
            "03. Boss Battle.ogg",
            "04. Mechanical Complex.ogg",
            "05. Last Mission.ogg",
            "06. Unknown Planet.ogg",
            "07. MonsterVania #1.ogg",
            "08. Space Adventure.ogg",
            "09. Crisis.ogg",
            "10. Jester Theme.ogg",
            "11. Jester Battle.ogg",
            "12. Strong Boss.ogg",
            "13. I am not clumsy.ogg",
            "14. MonsterVania #2.ogg",
            "15. Rush Point.ogg",
            "16. Truth.ogg",
            "17. The Quiet Spy.ogg",
            "18. Infinite Darkness.ogg",
        ]
    }

    fn extract_song_name(song_file: &str) -> String {
        song_file
            .trim_start_matches(|c: char| c.is_numeric() || c == '.' || c == ' ')
            .trim_end_matches(".ogg")
            .to_string()
    }

    /// Get the current loading state
    pub fn loading_state(&self) -> LoadingState {
        self.loading_state.lock().unwrap().clone()
    }

    /// Check if music has been loaded
    pub fn is_loaded(&self) -> bool {
        matches!(
            self.loading_state.lock().unwrap().clone(),
            LoadingState::Complete
        )
    }

    /// Start playing the playlist from the beginning
    pub fn start(&mut self) {
        if !self.muted {
            self.current_index = 0;
            self.play_current_song(false); // No fade-in on initial start
        }
    }

    /// Update the music manager - checks if current song finished and plays next
    pub fn update(&mut self) {
        // Don't update if muted or not loaded
        if self.muted || !self.is_loaded() {
            return;
        }

        // Check if current song has finished playing
        let should_play_next = if let Some(ref handle) = self.current_handle {
            handle.state() == PlaybackState::Stopped
        } else {
            true // No song playing, should start one
        };

        if should_play_next {
            self.play_next_song();
        }
    }
    
    /// Stop any currently playing song - ALWAYS call this before playing a new song
    fn stop_current_song(&mut self) {
        if let Some(mut handle) = self.current_handle.take() {
            let _ = handle.stop(Tween {
                duration: std::time::Duration::from_millis(500),
                ..Default::default()
            });
        }
    }

    /// Play the current song in the playlist with optional fade-in
    fn play_current_song(&mut self, fade_in: bool) {
        // ALWAYS stop current song first to ensure only one plays at a time
        self.stop_current_song();
        
        let playlist = self.playlist.lock().unwrap();
        if playlist.is_empty() {
            return;
        }

        let (name, sound_data) = &playlist[self.current_index];

        match self.audio_manager.play(sound_data.clone()) {
            Ok(handle) => {
                println!("Now playing: {}", name);
                self.current_handle = Some(handle);
            }
            Err(e) => {
                eprintln!("Failed to play {}: {}", name, e);
                self.current_handle = None;
            }
        }
    }

    /// Move to the next song in the playlist with smooth crossfade
    fn play_next_song(&mut self) {
        let playlist_len = self.playlist.lock().unwrap().len();
        if playlist_len == 0 {
            return;
        }

        // Crossfade handled by playing new song - no explicit fade needed

        // Move to next song and fade in
        self.current_index = (self.current_index + 1) % playlist_len;
        self.play_current_song(true);
    }

    /// Get the name of the currently playing song
    pub fn current_song_name(&self) -> String {
        let playlist = self.playlist.lock().unwrap();
        if playlist.is_empty() {
            return "No Music".to_string();
        }

        playlist[self.current_index].0.clone()
    }

    /// Skip to the next song
    pub fn skip_to_next(&mut self) {
        self.play_next_song();
    }

    /// Get the total number of songs in the playlist
    pub fn song_count(&self) -> usize {
        self.playlist.lock().unwrap().len()
    }

    /// Get the current song index (1-based)
    pub fn current_song_number(&self) -> usize {
        self.current_index + 1
    }
}

impl Default for MusicManager {
    fn default() -> Self {
        let volume_manager = VolumeManager::new();
        Self::new(volume_manager).expect("Failed to create music manager")
    }
}
