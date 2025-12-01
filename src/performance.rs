// src/performance.rs

use std::time::{Duration, Instant};

pub struct PerformanceMonitor {
    frame_count: u32,
    last_time: Instant,
    fps: f32,
    render_time: Duration,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        PerformanceMonitor {
            frame_count: 0,
            last_time: Instant::now(),
            fps: 0.0,
            render_time: Duration::new(0, 0),
        }
    }

    pub fn frame_rendered(&mut self, render_duration: Duration) {
        self.frame_count += 1;
        self.render_time = render_duration;

        if self.last_time.elapsed().as_secs() >= 1 {
            self.fps = self.frame_count as f32;
            self.frame_count = 0;
            self.last_time = Instant::now();
        }
    }

    pub fn get_fps(&self) -> f32 {
        self.fps
    }

    pub fn get_render_time(&self) -> Duration {
        self.render_time
    }

    // Here you can add more methods to fetch system stats if needed.
}