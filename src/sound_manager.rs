use crate::volume_manager::VolumeManager;
use kira::{AudioManager, DefaultBackend, Tween, sound::static_sound::StaticSoundData};
use std::sync::{Arc, Mutex};

/// Manages game sound effects (not music)
pub struct SoundManager {
    audio_manager: AudioManager<DefaultBackend>,
    sounds: Arc<Mutex<SoundEffects>>,
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
    /// Create a new sound manager (without loading sounds yet)
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

        Ok(Self {
            audio_manager,
            sounds: Arc::new(Mutex::new(SoundEffects {
                bounce: None,
                level_up: None,
                shuffle: None,
                success: None,
            })),
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
            if let Ok(sounds) = self.sounds.lock() {
                if let Some(ref sound) = sounds.bounce {
                    let _ = self.audio_manager.play(sound.clone());
                }
            }
        }
    }

    /// Play level up sound
    pub fn play_level_up(&mut self) {
        if !self.muted {
            if let Ok(sounds) = self.sounds.lock() {
                if let Some(ref sound) = sounds.level_up {
                    let _ = self.audio_manager.play(sound.clone());
                }
            }
        }
    }

    /// Play shuffle sound (piece rotates)
    pub fn play_shuffle(&mut self) {
        if !self.muted {
            if let Ok(sounds) = self.sounds.lock() {
                if let Some(ref sound) = sounds.shuffle {
                    let _ = self.audio_manager.play(sound.clone());
                }
            }
        }
    }

    /// Play success sound (lines cleared)
    pub fn play_success(&mut self) {
        if !self.muted {
            if let Ok(sounds) = self.sounds.lock() {
                if let Some(ref sound) = sounds.success {
                    let _ = self.audio_manager.play(sound.clone());
                }
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
            if let Ok(sounds) = self.sounds.lock() {
                if let Some(ref sound) = sounds.bounce {
                    let _ = self.audio_manager.play(sound.clone());
                }
            }
        }
    }
}
