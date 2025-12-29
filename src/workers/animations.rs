//! Animation system for workers

use std::time::Instant;

/// Animation controller
pub struct AnimationController {
    /// Start time of animation
    start_time: Instant,

    /// Frames per second
    fps: u32,

    /// Current frame index
    current_frame: u32,

    /// Total frames in animation
    total_frames: u32,

    /// Whether animation loops
    looping: bool,
}

impl AnimationController {
    pub fn new(fps: u32, total_frames: u32, looping: bool) -> Self {
        Self {
            start_time: Instant::now(),
            fps,
            current_frame: 0,
            total_frames,
            looping,
        }
    }

    /// Update and get current frame
    pub fn tick(&mut self) -> u32 {
        let elapsed = self.start_time.elapsed().as_millis() as u32;
        let frame_duration = 1000 / self.fps;

        let new_frame = elapsed / frame_duration;

        if self.looping {
            self.current_frame = new_frame % self.total_frames;
        } else {
            self.current_frame = new_frame.min(self.total_frames - 1);
        }

        self.current_frame
    }

    /// Check if non-looping animation is complete
    pub fn is_complete(&self) -> bool {
        !self.looping && self.current_frame >= self.total_frames - 1
    }

    /// Reset animation
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        self.current_frame = 0;
    }
}

/// Easing functions for smooth animations
pub mod easing {
    /// Linear interpolation
    pub fn linear(t: f64) -> f64 {
        t
    }

    /// Ease in (slow start)
    pub fn ease_in_quad(t: f64) -> f64 {
        t * t
    }

    /// Ease out (slow end)
    pub fn ease_out_quad(t: f64) -> f64 {
        1.0 - (1.0 - t) * (1.0 - t)
    }

    /// Ease in and out
    pub fn ease_in_out_quad(t: f64) -> f64 {
        if t < 0.5 {
            2.0 * t * t
        } else {
            1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
        }
    }

    /// Bounce effect
    pub fn bounce(t: f64) -> f64 {
        let n1 = 7.5625;
        let d1 = 2.75;

        if t < 1.0 / d1 {
            n1 * t * t
        } else if t < 2.0 / d1 {
            let t = t - 1.5 / d1;
            n1 * t * t + 0.75
        } else if t < 2.5 / d1 {
            let t = t - 2.25 / d1;
            n1 * t * t + 0.9375
        } else {
            let t = t - 2.625 / d1;
            n1 * t * t + 0.984375
        }
    }
}

/// Particle effect for celebrations
pub struct Particle {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub char: char,
    pub lifetime: u32,
    pub age: u32,
}

impl Particle {
    pub fn new(x: f64, y: f64, char: char, lifetime: u32) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        Self {
            x,
            y,
            vx: rng.gen_range(-2.0..2.0),
            vy: rng.gen_range(-3.0..-0.5),
            char,
            lifetime,
            age: 0,
        }
    }

    pub fn tick(&mut self) {
        self.x += self.vx;
        self.y += self.vy;
        self.vy += 0.1; // gravity
        self.age += 1;
    }

    pub fn is_alive(&self) -> bool {
        self.age < self.lifetime
    }
}

/// Particle system for effects
pub struct ParticleSystem {
    particles: Vec<Particle>,
    max_particles: usize,
}

impl ParticleSystem {
    pub fn new(max_particles: usize) -> Self {
        Self {
            particles: Vec::with_capacity(max_particles),
            max_particles,
        }
    }

    /// Emit particles at position
    pub fn emit(&mut self, x: f64, y: f64, count: usize) {
        let chars = ['✦', '✧', '⋆', '·', '★', '☆'];

        for i in 0..count {
            if self.particles.len() >= self.max_particles {
                break;
            }

            let char = chars[i % chars.len()];
            self.particles.push(Particle::new(x, y, char, 30));
        }
    }

    /// Update all particles
    pub fn tick(&mut self) {
        for particle in &mut self.particles {
            particle.tick();
        }

        self.particles.retain(|p| p.is_alive());
    }

    /// Get active particles
    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }
}
