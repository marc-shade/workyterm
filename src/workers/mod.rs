//! AI Worker characters and office simulation

mod sprites;
mod animations;

pub use sprites::*;
pub use animations::*;

use crate::config::Config;

/// Worker state in the virtual office
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerState {
    /// Waiting for work
    Idle,
    /// Processing/thinking about the task
    Thinking,
    /// Actively generating output
    Typing,
    /// Task completed successfully
    Celebrating,
    /// Encountered an error or confusion
    Confused,
    /// Working with other workers
    Collaborating,
}

/// An AI worker character
#[derive(Debug, Clone)]
pub struct Worker {
    /// Worker's display name
    pub name: String,

    /// Current state
    pub state: WorkerState,

    /// Associated LLM provider
    pub provider: String,

    /// Current animation frame
    pub frame: u8,

    /// Position in office (for future use)
    pub position: (u16, u16),

    /// Worker's specialty
    pub specialty: WorkerSpecialty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerSpecialty {
    Writer,
    Researcher,
    Analyst,
    Creative,
    Editor,
}

impl Worker {
    pub fn new(name: String, provider: String, specialty: WorkerSpecialty) -> Self {
        Self {
            name,
            state: WorkerState::Idle,
            provider,
            frame: 0,
            position: (0, 0),
            specialty,
        }
    }

    /// Update worker animation
    pub fn tick(&mut self, tick: u64) {
        self.frame = ((tick / 5) % 4) as u8;
    }
}

/// The virtual office containing workers
#[derive(Debug)]
pub struct Office {
    pub workers: Vec<Worker>,
}

impl Office {
    pub fn new(config: &Config) -> Self {
        let specialties = [
            WorkerSpecialty::Writer,
            WorkerSpecialty::Researcher,
            WorkerSpecialty::Analyst,
            WorkerSpecialty::Creative,
            WorkerSpecialty::Editor,
        ];

        let workers: Vec<Worker> = config
            .ui
            .worker_names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let provider = config
                    .council
                    .members
                    .get(i % config.council.members.len())
                    .cloned()
                    .unwrap_or_else(|| config.default_provider.clone());

                Worker::new(
                    name.clone(),
                    provider,
                    specialties[i % specialties.len()],
                )
            })
            .collect();

        Self { workers }
    }

    /// Update all worker animations
    pub fn tick(&mut self, tick: u64) {
        for worker in &mut self.workers {
            worker.tick(tick);
        }
    }

    /// Get workers by state
    pub fn workers_in_state(&self, state: WorkerState) -> Vec<&Worker> {
        self.workers.iter().filter(|w| w.state == state).collect()
    }
}
