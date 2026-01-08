use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeSettings {
    pub music_volume: f32,
    pub sfx_volume: f32,
}

impl Default for VolumeSettings {
    fn default() -> Self {
        Self {
            music_volume: 0.5,
            sfx_volume: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameData {
    pub high_score: u64,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            high_score: 0,
        }
    }
}

/// Platform-agnostic storage for game settings
pub struct Storage;

impl Storage {
    const VOLUME_KEY: &'static str = "retris_volume_settings";
    const GAME_DATA_KEY: &'static str = "retris_game_data";
    
    /// Load volume settings from storage (localStorage on web, file on native)
    pub fn load_volume() -> VolumeSettings {
        #[cfg(target_arch = "wasm32")]
        {
            Self::load_volume_web().unwrap_or_default()
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::load_volume_native().unwrap_or_default()
        }
    }
    
    /// Check if volume settings exist in storage (returns true if settings were previously saved)
    pub fn has_volume_settings() -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            Self::load_volume_web().is_some()
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::load_volume_native().is_some()
        }
    }
    
    /// Save volume settings to storage
    pub fn save_volume(settings: &VolumeSettings) {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = Self::save_volume_web(settings);
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = Self::save_volume_native(settings);
        }
    }
    
    /// Load game data from storage (high score, etc.)
    pub fn load_game_data() -> GameData {
        #[cfg(target_arch = "wasm32")]
        {
            Self::load_game_data_web().unwrap_or_default()
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::load_game_data_native().unwrap_or_default()
        }
    }
    
    /// Save game data to storage
    pub fn save_game_data(data: &GameData) {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = Self::save_game_data_web(data);
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = Self::save_game_data_native(data);
        }
    }
    
    // ===== Web implementation (localStorage) =====
    
    #[cfg(target_arch = "wasm32")]
    fn load_volume_web() -> Option<VolumeSettings> {
        use web_sys::window;
        
        let window = window()?;
        let storage = window.local_storage().ok()??;
        let json = storage.get_item(Self::VOLUME_KEY).ok()??;
        
        serde_json::from_str(&json).ok()
    }
    
    #[cfg(target_arch = "wasm32")]
    fn save_volume_web(settings: &VolumeSettings) -> Result<(), String> {
        use web_sys::window;
        
        let window = window().ok_or("No window")?;
        let storage = window.local_storage()
            .map_err(|_| "No localStorage")?
            .ok_or("No localStorage")?;
        
        let json = serde_json::to_string(settings)
            .map_err(|e| format!("Serialize error: {}", e))?;
        
        storage.set_item(Self::VOLUME_KEY, &json)
            .map_err(|_| "Failed to set item".to_string())?;
        
        println!("Saved volume settings to localStorage");
        Ok(())
    }
    
    #[cfg(target_arch = "wasm32")]
    fn load_game_data_web() -> Option<GameData> {
        use web_sys::window;
        
        let window = window()?;
        let storage = window.local_storage().ok()??;
        let json = storage.get_item(Self::GAME_DATA_KEY).ok()??;
        
        serde_json::from_str(&json).ok()
    }
    
    #[cfg(target_arch = "wasm32")]
    fn save_game_data_web(data: &GameData) -> Result<(), String> {
        use web_sys::window;
        
        let window = window().ok_or("No window")?;
        let storage = window.local_storage()
            .map_err(|_| "No localStorage")?
            .ok_or("No localStorage")?;
        
        let json = serde_json::to_string(data)
            .map_err(|e| format!("Serialize error: {}", e))?;
        
        storage.set_item(Self::GAME_DATA_KEY, &json)
            .map_err(|_| "Failed to set item".to_string())?;
        
        println!("Saved game data to localStorage");
        Ok(())
    }
    
    // ===== Native implementation (config file) =====
    
    #[cfg(not(target_arch = "wasm32"))]
    fn config_path() -> Option<std::path::PathBuf> {
        // Try XDG config dir first (Linux/macOS)
        if let Ok(config_dir) = std::env::var("XDG_CONFIG_HOME") {
            let mut path = std::path::PathBuf::from(config_dir);
            path.push("retris");
            return Some(path);
        }
        
        // Fallback to home directory
        if let Ok(home) = std::env::var("HOME") {
            let mut path = std::path::PathBuf::from(home);
            path.push(".config");
            path.push("retris");
            return Some(path);
        }
        
        None
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn load_volume_native() -> Option<VolumeSettings> {
        let mut path = Self::config_path()?;
        path.push("settings.json");
        
        let contents = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&contents).ok()
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn save_volume_native(settings: &VolumeSettings) -> Result<(), String> {
        let config_dir = Self::config_path().ok_or("No config directory")?;
        
        // Create config directory if it doesn't exist
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config dir: {}", e))?;
        
        let mut path = config_dir;
        path.push("settings.json");
        
        let json = serde_json::to_string_pretty(settings)
            .map_err(|e| format!("Serialize error: {}", e))?;
        
        std::fs::write(&path, json)
            .map_err(|e| format!("Failed to write file: {}", e))?;
        
        println!("Saved volume settings to {:?}", path);
        Ok(())
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn load_game_data_native() -> Option<GameData> {
        let mut path = Self::config_path()?;
        path.push("game_data.json");
        
        let contents = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&contents).ok()
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn save_game_data_native(data: &GameData) -> Result<(), String> {
        let config_dir = Self::config_path().ok_or("No config directory")?;
        
        // Create config directory if it doesn't exist
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config dir: {}", e))?;
        
        let mut path = config_dir;
        path.push("game_data.json");
        
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| format!("Serialize error: {}", e))?;
        
        std::fs::write(&path, json)
            .map_err(|e| format!("Failed to write file: {}", e))?;
        
        println!("Saved game data to {:?}", path);
        Ok(())
    }
}
