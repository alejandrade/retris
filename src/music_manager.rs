use crate::volume_manager::VolumeManager;
use kira::{
    AudioManager, AudioManagerSettings, DefaultBackend, Tween,
    sound::{
        PlaybackState,
        static_sound::{StaticSoundData, StaticSoundHandle},
    },
};
use std::io::Cursor;
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
    // Store song names and raw bytes - decode on demand
    // Use Vec<u8> instead of &'static [u8] for WASM compatibility
    song_bytes: Vec<(String, Vec<u8>)>,
    song_names: Vec<String>,
    // Currently decoded song (unloaded when next song loads)
    current_decoded_song: Option<StaticSoundData>,
    current_index: usize,
    current_handle: Option<StaticSoundHandle>,
    loading_state: Arc<Mutex<LoadingState>>,
    muted: bool,
    volume_manager: VolumeManager,
    // Game over sound - decode on demand
    game_over_bytes: Vec<u8>,
    game_over_decoded: Option<StaticSoundData>,
    game_over_handle: Option<StaticSoundHandle>,
    // New flag to control if the playlist is allowed to advance
    playlist_active: bool,
    // Track if audio context has been resumed (needed for WASM/browser autoplay policy)
    audio_context_resumed: bool,
}

impl MusicManager {
    pub fn new(volume_manager: VolumeManager) -> Result<Self, Box<dyn std::error::Error>> {
        let mut audio_manager =
            AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())?;

        let initial_volume = volume_manager.music_volume();
        let db = Self::amplitude_to_db(initial_volume);
        let _ = audio_manager.main_track().set_volume(db, Tween::default());

        println!(
            "MusicManager initialized with volume {} ({:.1} dB)",
            initial_volume, db
        );

        // Get song metadata immediately (no decoding)
        // Convert to owned data for WASM compatibility
        let song_bytes: Vec<(String, Vec<u8>)> = vec![
            (
                Self::extract_song_name("01. Slay The Evil.ogg"),
                include_bytes!("../assets/01. Slay The Evil.ogg").to_vec(),
            ),
            (
                Self::extract_song_name("02. Perilous Dungeon.ogg"),
                include_bytes!("../assets/02. Perilous Dungeon.ogg").to_vec(),
            ),
            (
                Self::extract_song_name("03. Boss Battle.ogg"),
                include_bytes!("../assets/03. Boss Battle.ogg").to_vec(),
            ),
            (
                Self::extract_song_name("04. Mechanical Complex.ogg"),
                include_bytes!("../assets/04. Mechanical Complex.ogg").to_vec(),
            ),
            (
                Self::extract_song_name("05. Last Mission.ogg"),
                include_bytes!("../assets/05. Last Mission.ogg").to_vec(),
            ),
            (
                Self::extract_song_name("06. Unknown Planet.ogg"),
                include_bytes!("../assets/06. Unknown Planet.ogg").to_vec(),
            ),
            (
                Self::extract_song_name("07. MonsterVania #1.ogg"),
                include_bytes!("../assets/07. MonsterVania #1.ogg").to_vec(),
            ),
            (
                Self::extract_song_name("08. Space Adventure.ogg"),
                include_bytes!("../assets/08. Space Adventure.ogg").to_vec(),
            ),
            (
                Self::extract_song_name("09. Crisis.ogg"),
                include_bytes!("../assets/09. Crisis.ogg").to_vec(),
            ),
            (
                Self::extract_song_name("10. Jester Theme.ogg"),
                include_bytes!("../assets/10. Jester Theme.ogg").to_vec(),
            ),
        ];
        let song_names: Vec<String> = song_bytes.iter().map(|(name, _)| name.clone()).collect();

        let game_over_bytes =
            include_bytes!("../assets/219117__stanrams__trumpet-game-over-baby.ogg").to_vec();

        Ok(Self {
            audio_manager,
            song_bytes,
            song_names,
            current_decoded_song: None,
            current_index: 0,
            current_handle: None,
            loading_state: Arc::new(Mutex::new(LoadingState::Complete)),
            muted: false,
            volume_manager,
            game_over_bytes,
            game_over_decoded: None,
            game_over_handle: None,
            playlist_active: false,
            audio_context_resumed: false,
        })
    }

    fn amplitude_to_db(amplitude: f32) -> f32 {
        if amplitude <= 0.0 {
            -60.0
        } else {
            let curved = amplitude * amplitude;
            20.0 * curved.log10()
        }
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
        if muted {
            self.stop_current_song();
            self.stop_game_over_song();
        }
    }

    pub fn is_muted(&self) -> bool {
        self.muted
    }

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

    /// Resume audio context (required for WASM/browser autoplay policy)
    /// Should be called on first user interaction. According to Chrome docs, the AudioContext
    /// will be resumed after a user gesture if start() is called on any attached node.
    /// Playing a sound will trigger the resume automatically.
    pub fn ensure_audio_context_resumed(&mut self) {
        if !self.audio_context_resumed {
            // Mark as resumed - actual resume will happen when we try to play
            // The AudioContext was warmed up during startup, so it should resume
            // when we call play() after a user gesture
            self.audio_context_resumed = true;
        }
    }

    // --- Loading Logic ---

    /// Helper to load audio data from embedded bytes
    fn load_audio_data_from_bytes(
        bytes: &[u8],
    ) -> Result<StaticSoundData, Box<dyn std::error::Error>> {
        // Clone bytes to ensure they live long enough for decoding
        let bytes_vec = bytes.to_vec();
        Ok(StaticSoundData::from_cursor(Cursor::new(bytes_vec))?)
    }

    pub fn start_loading_background(&mut self) {
        // Lazy loading: songs are already indexed, no decoding needed yet
        // However, we decode one song at startup then immediately unload it
        // This "warms up" the AudioContext during initialization, which helps
        // with browser autoplay policy (similar to how it worked before)
        if !self.song_bytes.is_empty() {
            // Decode the first song to warm up AudioContext
            let (filename, bytes) = &self.song_bytes[0];
            if let Ok(sound_data) = Self::load_audio_data_from_bytes(bytes) {
                println!("Warmed up AudioContext with: {} (now unloading)", filename);
                // Immediately drop it - this was just to initialize the AudioContext
                // The actual decoding happens when we need to play
                drop(sound_data);
            }
        }

        let total = self.song_bytes.len();
        *self.loading_state.lock().unwrap() = LoadingState::Complete;
        println!(
            "Music manager ready: {} songs available (will load on demand)",
            total
        );
    }

    // --- Playback Logic ---

    pub fn start(&mut self) {
        if !self.muted {
            // Requirement: Start main playlist stops game over
            self.stop_game_over_song();

            self.current_index = 0;
            self.playlist_active = true; // Enable playlist progression
            self.play_current_song(false);
        }
    }

    pub fn update(&mut self) {
        if self.muted || !self.is_loaded() {
            return;
        }

        // 1. Check Game Over Logic
        if let Some(ref handle) = self.game_over_handle {
            // If game over is playing, do nothing else.
            if handle.state() == PlaybackState::Playing {
                return;
            }
            // If it stopped, just clear the handle.
            // Requirement: "Once game over ends it should just be silent"
            // We do NOT set playlist_active to true here.
            self.game_over_handle = None;
        }

        // 2. Check Playlist Logic
        // If the playlist isn't active (e.g., game over happened), don't play next song.
        if !self.playlist_active {
            return;
        }

        // Check if current playlist song has finished
        let song_finished = if let Some(ref handle) = self.current_handle {
            handle.state() == PlaybackState::Stopped
        } else {
            true // No song playing, but playlist is active, so start one
        };

        if song_finished {
            self.play_next_song();
        }
    }

    /// Stops the playlist music
    fn stop_current_song(&mut self) {
        if let Some(mut handle) = self.current_handle.take() {
            let _ = handle.stop(Tween {
                duration: std::time::Duration::from_millis(500),
                ..Default::default()
            });
        }
    }

    /// Stops the game over sound specifically
    fn stop_game_over_song(&mut self) {
        if let Some(mut handle) = self.game_over_handle.take() {
            let _ = handle.stop(Tween {
                duration: std::time::Duration::from_millis(100),
                ..Default::default()
            });
        }
    }

    pub fn play_game_over_song(&mut self) {
        // Requirement: Game over stops playlist
        self.stop_current_song();

        // Unload current song
        self.current_decoded_song = None;

        // Requirement: Game over shouldn't be part of main playlist loop
        self.playlist_active = false;

        // Stop any existing game over sound before playing new one
        self.stop_game_over_song();

        // Load game over sound on demand
        if self.game_over_decoded.is_none() {
            println!("Decoding game over sound");
            match Self::load_audio_data_from_bytes(&self.game_over_bytes) {
                Ok(data) => {
                    self.game_over_decoded = Some(data);
                }
                Err(e) => {
                    eprintln!("Failed to decode game over song: {}", e);
                    return;
                }
            }
        }

        if let Some(ref sound_data) = self.game_over_decoded {
            match self.audio_manager.play(sound_data.clone()) {
                Ok(handle) => {
                    println!("Playing game over song");
                    self.game_over_handle = Some(handle);
                }
                Err(e) => eprintln!("Failed to play game over song: {}", e),
            }
        }
    }

    fn play_current_song(&mut self, _fade_in: bool) {
        self.stop_current_song();

        if self.song_bytes.is_empty() || self.current_index >= self.song_bytes.len() {
            return;
        }

        // Unload previous song (free memory)
        self.current_decoded_song = None;

        // Load current song on demand
        let (filename, bytes) = &self.song_bytes[self.current_index];
        println!("Decoding: {}", filename);

        match Self::load_audio_data_from_bytes(bytes) {
            Ok(sound_data) => {
                // Keep decoded song until next one loads
                let name = &self.song_names[self.current_index];
                match self.audio_manager.play(sound_data.clone()) {
                    Ok(handle) => {
                        println!("Now playing: {}", name);
                        self.current_handle = Some(handle);
                        self.current_decoded_song = Some(sound_data);
                    }
                    Err(e) => {
                        eprintln!("Failed to play {}: {}", name, e);
                        self.current_handle = None;
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to decode {}: {}", filename, e);
                self.current_handle = None;
            }
        }
    }

    fn play_next_song(&mut self) {
        if self.song_bytes.is_empty() {
            return;
        }
        self.current_index = (self.current_index + 1) % self.song_bytes.len();
        self.play_current_song(true);
    }

    pub fn skip_to_next(&mut self) {
        // Only allow skipping if we are in "Playlist Mode"
        if self.playlist_active {
            self.play_next_song();
        }
    }

    // --- Data Helpers ---

    /// Get all song data as (filename, bytes) tuples
    fn get_song_data() -> Vec<(&'static str, &'static [u8])> {
        vec![
            (
                "01. Slay The Evil.ogg",
                include_bytes!("../assets/01. Slay The Evil.ogg"),
            ),
            (
                "02. Perilous Dungeon.ogg",
                include_bytes!("../assets/02. Perilous Dungeon.ogg"),
            ),
            (
                "03. Boss Battle.ogg",
                include_bytes!("../assets/03. Boss Battle.ogg"),
            ),
            (
                "04. Mechanical Complex.ogg",
                include_bytes!("../assets/04. Mechanical Complex.ogg"),
            ),
            (
                "05. Last Mission.ogg",
                include_bytes!("../assets/05. Last Mission.ogg"),
            ),
            (
                "06. Unknown Planet.ogg",
                include_bytes!("../assets/06. Unknown Planet.ogg"),
            ),
            (
                "07. MonsterVania #1.ogg",
                include_bytes!("../assets/07. MonsterVania #1.ogg"),
            ),
            (
                "08. Space Adventure.ogg",
                include_bytes!("../assets/08. Space Adventure.ogg"),
            ),
            ("09. Crisis.ogg", include_bytes!("../assets/09. Crisis.ogg")),
            (
                "10. Jester Theme.ogg",
                include_bytes!("../assets/10. Jester Theme.ogg"),
            ),
        ]
    }

    fn extract_song_name(song_file: &str) -> String {
        song_file
            .trim_start_matches(|c: char| c.is_numeric() || c == '.' || c == ' ')
            .trim_end_matches(".ogg")
            .to_string()
    }

    pub fn loading_state(&self) -> LoadingState {
        self.loading_state.lock().unwrap().clone()
    }

    pub fn is_loaded(&self) -> bool {
        // Always ready since we load on-demand
        true
    }

    pub fn stop(&mut self) {
        self.stop_current_song();
        self.stop_game_over_song();
    }

    pub fn current_song_name(&self) -> String {
        if self.song_names.is_empty() || self.current_index >= self.song_names.len() {
            return "No Music".to_string();
        }
        self.song_names[self.current_index].clone()
    }

    pub fn song_count(&self) -> usize {
        self.song_bytes.len()
    }

    pub fn current_song_number(&self) -> usize {
        self.current_index + 1
    }

    // Test sound methods kept but ensured they affect state correctly
    pub fn test_sound(&mut self) {
        self.stop_current_song();
        self.playlist_active = false; // Test sound is not the playlist

        if !self.song_bytes.is_empty() {
            // Load first song for testing
            let (filename, bytes) = &self.song_bytes[0];
            match Self::load_audio_data_from_bytes(bytes) {
                Ok(sound_data) => {
                    if let Ok(handle) = self.audio_manager.play(sound_data.clone()) {
                        self.current_handle = Some(handle);
                        // Keep decoded for test playback
                        self.current_decoded_song = Some(sound_data);
                    }
                }
                Err(e) => eprintln!("Failed to decode {} for test: {}", filename, e),
            }
        }
    }

    pub fn stop_test_sound(&mut self) {
        self.stop_current_song();
    }
}

impl Default for MusicManager {
    fn default() -> Self {
        let volume_manager = VolumeManager::new();
        Self::new(volume_manager).expect("Failed to create music manager")
    }
}
