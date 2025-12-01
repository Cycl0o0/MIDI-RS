// src/config.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub width: u32,
    pub height: u32,
    pub target_fps: u32,
    pub background_color: [f32; 4],
    pub note_width: f32,
    pub note_height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityConfig {
    pub max_note_count: u32,
    pub use_instancing: bool,
    pub frustum_culling: bool,
    pub particle_density: f32,
    pub effect_quality: EffectQuality,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum EffectQuality {
    Low,
    Medium,
    High,
    Ultra,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub enable_performance_overlay: bool,
    pub slow_mode: bool,
    pub playback_speed: f32,
    pub frame_lock: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiConfig {
    pub streaming_enabled: bool,
    pub buffer_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub display: DisplayConfig,
    pub quality: QualityConfig,
    pub performance: PerformanceConfig,
    pub midi: MidiConfig,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        DisplayConfig {
            width: 1920,
            height: 1080,
            target_fps: 60,
            background_color: [0.05, 0.05, 0.05, 1.0],
            note_width: 2.0,
            note_height: 0.15,
        }
    }
}

impl Default for QualityConfig {
    fn default() -> Self {
        QualityConfig {
            max_note_count: 1_000_000,
            use_instancing: true,
            frustum_culling: true,
            particle_density: 1.0,
            effect_quality: EffectQuality::High,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        PerformanceConfig {
            enable_performance_overlay: false,
            slow_mode: false,
            playback_speed: 1.0,
            frame_lock: None,
        }
    }
}

impl Default for MidiConfig {
    fn default() -> Self {
        MidiConfig {
            streaming_enabled: true,
            buffer_size: 65536,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            display: DisplayConfig::default(),
            quality: QualityConfig::default(),
            performance: PerformanceConfig::default(),
            midi: MidiConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn slow_mode_config() -> Self {
        let mut config = AppConfig::default();
        config.performance.slow_mode = true;
        config.performance.frame_lock = Some(30);
        config.quality.particle_density = 0.5;
        config
    }

    pub fn performance_mode_config() -> Self {
        let mut config = AppConfig::default();
        config.performance.slow_mode = false;
        config.performance.frame_lock = Some(144);
        config.quality.effect_quality = EffectQuality::Ultra;
        config
    }
}