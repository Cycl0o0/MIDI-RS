struct Note {
    pitch: u8,
    velocity: u8,
    start_time: f32,
    duration: f32,
    channel: u8,
}

impl Note {
    // Calculate end time of the note
    fn end_time(&self) -> f32 {
        self.start_time + self.duration
    }

    // Get color based on the channel using HSV to RGB conversion
    fn get_color(&self) -> (u8, u8, u8) {
        let (h, s, v) = (self.channel as f32 * 60.0, 255.0, 255.0);
        let c = v * s / 255.0;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v as f32 - c;
        let (r, g, b) = match h {
            0.0..=60.0 => (c, x, 0.0),
            60.0..=120.0 => (x, c, 0.0),
            120.0..=180.0 => (0.0, c, x),
            180.0..=240.0 => (0.0, x, c),
            240.0..=300.0 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        ((r + m) as u8, (g + m) as u8, (b + m) as u8)
    }

    // Get y position for screen rendering
    fn get_y_position(&self, screen_height: f32) -> f32 {
        screen_height - (self.channel as f32 * (screen_height / 16.0))
    }
}