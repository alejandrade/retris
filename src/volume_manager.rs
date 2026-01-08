use crate::storage::Storage;
use std::sync::{Arc, Mutex};

/// Centralized volume control for all audio
#[derive(Clone)]
pub struct VolumeManager {
    inner: Arc<Mutex<VolumeSettings>>,
}

struct VolumeSettings {
    music_volume: f32,  // 0.0 to 1.0
    sfx_volume: f32,    // 0.0 to 1.0
}

impl VolumeManager {
    /// Create a new volume manager, loading from storage if available
    pub fn new() -> Self {
        let settings = Storage::load_volume();
        println!("Loaded volume settings: music={}, sfx={}", settings.music_volume, settings.sfx_volume);
        
        Self {
            inner: Arc::new(Mutex::new(VolumeSettings {
                music_volume: settings.music_volume,
                sfx_volume: settings.sfx_volume,
            })),
        }
    }
    
    /// Get the current music volume (0.0 to 1.0)
    pub fn music_volume(&self) -> f32 {
        self.inner.lock().unwrap().music_volume
    }
    
    /// Get the current sound effects volume (0.0 to 1.0)
    pub fn sfx_volume(&self) -> f32 {
        self.inner.lock().unwrap().sfx_volume
    }
    
    /// Set music volume (0.0 to 1.0) - does NOT auto-save
    pub fn set_music_volume(&self, volume: f32) {
        self.inner.lock().unwrap().music_volume = volume.clamp(0.0, 1.0);
    }
    
    /// Set sound effects volume (0.0 to 1.0) - does NOT auto-save
    pub fn set_sfx_volume(&self, volume: f32) {
        self.inner.lock().unwrap().sfx_volume = volume.clamp(0.0, 1.0);
    }
    
    /// Save current settings to storage (call this explicitly when ready to persist)
    pub fn save(&self) {
        let settings = self.inner.lock().unwrap();
        Storage::save_volume(&crate::storage::VolumeSettings {
            music_volume: settings.music_volume,
            sfx_volume: settings.sfx_volume,
        });
    }
}

impl Default for VolumeManager {
    fn default() -> Self {
        Self::new()
    }
}
