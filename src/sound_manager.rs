use crate::volume_manager::VolumeManager;
use kira::{AudioManager, DefaultBackend, Tween, sound::static_sound::StaticSoundData};
use std::io::Cursor;

/// Manages game sound effects (not music)
pub struct SoundManager {
    audio_manager: AudioManager<DefaultBackend>,
    sounds: SoundEffects,
    muted: bool,
    volume_manager: VolumeManager,
}

struct SoundEffects {
    bounce: Option<StaticSoundData>,
    level_up: Option<StaticSoundData>,
    shuffle: Option<StaticSoundData>,
    success: Option<StaticSoundData>,
}

impl SoundManager {
    /// Create a new sound manager and load all sound effects
    pub fn new(volume_manager: VolumeManager) -> Result<Self, Box<dyn std::error::Error>> {
        let mut audio_manager = AudioManager::<DefaultBackend>::new(Default::default())?;

        // Set initial volume
        let initial_volume = volume_manager.sfx_volume();
        let db = Self::amplitude_to_db(initial_volume);
        let _ = audio_manager.main_track().set_volume(db, Tween::default());

        println!(
            "SoundManager initialized with volume {} ({:.1} dB)",
            initial_volume, db
        );

        // Load sound effects
        let bounce = Self::load_audio_data_from_bytes(include_bytes!("../assets/bounce.ogg")).ok();
        let level_up = Self::load_audio_data_from_bytes(include_bytes!("../assets/level-up.ogg")).ok();
        let shuffle = Self::load_audio_data_from_bytes(include_bytes!("../assets/shufle.ogg")).ok(); // Note: filename has typo "shufle"
        let success = Self::load_audio_data_from_bytes(include_bytes!("../assets/success.ogg")).ok();

        if bounce.is_some() {
            println!("Loaded bounce sound");
        } else {
            eprintln!("Failed to load bounce sound");
        }
        if level_up.is_some() {
            println!("Loaded level-up sound");
        } else {
            eprintln!("Failed to load level-up sound");
        }
        if shuffle.is_some() {
            println!("Loaded shuffle sound");
        } else {
            eprintln!("Failed to load shuffle sound");
        }
        if success.is_some() {
            println!("Loaded success sound");
        } else {
            eprintln!("Failed to load success sound");
        }

        Ok(Self {
            audio_manager,
            sounds: SoundEffects {
                bounce,
                level_up,
                shuffle,
                success,
            },
            muted: false,
            volume_manager,
        })
    }

    /// Helper to load audio data from embedded bytes
    fn load_audio_data_from_bytes(
        bytes: &[u8],
    ) -> Result<StaticSoundData, Box<dyn std::error::Error>> {
        // Clone bytes to ensure they live long enough for decoding
        let bytes_vec = bytes.to_vec();
        Ok(StaticSoundData::from_cursor(Cursor::new(bytes_vec))?)
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

    /// Update volume from VolumeManager
    pub fn update_volume(&mut self) {
        let volume = self.volume_manager.sfx_volume();
        let db = Self::amplitude_to_db(volume);
        let _ = self.audio_manager.main_track().set_volume(
            db,
            Tween {
                duration: std::time::Duration::from_millis(100),
                ..Default::default()
            },
        );
    }

    /// Play bounce sound (piece lands)
    pub fn play_bounce(&mut self) {
        if !self.muted {
            if let Some(ref sound) = self.sounds.bounce {
                let _ = self.audio_manager.play(sound.clone());
            }
        }
    }

    /// Play level up sound
    pub fn play_level_up(&mut self) {
        if !self.muted {
            if let Some(ref sound) = self.sounds.level_up {
                let _ = self.audio_manager.play(sound.clone());
            }
        }
    }

    /// Play shuffle sound (piece rotates)
    pub fn play_shuffle(&mut self) {
        if !self.muted {
            if let Some(ref sound) = self.sounds.shuffle {
                let _ = self.audio_manager.play(sound.clone());
            }
        }
    }

    /// Play success sound (lines cleared)
    pub fn play_success(&mut self) {
        if !self.muted {
            if let Some(ref sound) = self.sounds.success {
                let _ = self.audio_manager.play(sound.clone());
            }
        }
    }

    /// Set whether sound effects are muted
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }

    /// Play a test sound (plays bounce sound)
    pub fn test_sound(&mut self) {
        if !self.muted {
            if let Some(ref sound) = self.sounds.bounce {
                let _ = self.audio_manager.play(sound.clone());
            }
        }
    }
}
