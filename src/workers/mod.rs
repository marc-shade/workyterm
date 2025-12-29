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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_creation() {
        let worker = Worker::new(
            "TestWorker".to_string(),
            "ollama".to_string(),
            WorkerSpecialty::Writer,
        );

        assert_eq!(worker.name, "TestWorker");
        assert_eq!(worker.provider, "ollama");
        assert_eq!(worker.state, WorkerState::Idle);
        assert_eq!(worker.frame, 0);
        assert_eq!(worker.position, (0, 0));
        assert_eq!(worker.specialty, WorkerSpecialty::Writer);
    }

    #[test]
    fn test_worker_tick() {
        let mut worker = Worker::new(
            "TestWorker".to_string(),
            "ollama".to_string(),
            WorkerSpecialty::Writer,
        );

        // Frame should cycle through 0-3
        worker.tick(0);
        assert_eq!(worker.frame, 0);

        worker.tick(5);
        assert_eq!(worker.frame, 1);

        worker.tick(10);
        assert_eq!(worker.frame, 2);

        worker.tick(15);
        assert_eq!(worker.frame, 3);

        worker.tick(20);
        assert_eq!(worker.frame, 0); // Should cycle back
    }

    #[test]
    fn test_office_creation() {
        let config = Config::default();
        let office = Office::new(&config);

        assert_eq!(office.workers.len(), 5);

        // Check worker names match config
        let worker_names: Vec<&str> = office.workers.iter().map(|w| w.name.as_str()).collect();
        assert!(worker_names.contains(&"Pixel"));
        assert!(worker_names.contains(&"Byte"));
        assert!(worker_names.contains(&"Nova"));
        assert!(worker_names.contains(&"Chip"));
        assert!(worker_names.contains(&"Luna"));
    }

    #[test]
    fn test_office_tick() {
        let config = Config::default();
        let mut office = Office::new(&config);

        // All workers should start at frame 0
        for worker in &office.workers {
            assert_eq!(worker.frame, 0);
        }

        // After tick, all workers should update
        office.tick(5);
        for worker in &office.workers {
            assert_eq!(worker.frame, 1);
        }
    }

    #[test]
    fn test_workers_in_state() {
        let config = Config::default();
        let mut office = Office::new(&config);

        // All workers start idle
        let idle_workers = office.workers_in_state(WorkerState::Idle);
        assert_eq!(idle_workers.len(), 5);

        // Change one worker to thinking
        office.workers[0].state = WorkerState::Thinking;

        let idle_workers = office.workers_in_state(WorkerState::Idle);
        assert_eq!(idle_workers.len(), 4);

        let thinking_workers = office.workers_in_state(WorkerState::Thinking);
        assert_eq!(thinking_workers.len(), 1);
    }

    #[test]
    fn test_worker_state_transitions() {
        let mut worker = Worker::new(
            "TestWorker".to_string(),
            "ollama".to_string(),
            WorkerSpecialty::Analyst,
        );

        assert_eq!(worker.state, WorkerState::Idle);

        worker.state = WorkerState::Thinking;
        assert_eq!(worker.state, WorkerState::Thinking);

        worker.state = WorkerState::Typing;
        assert_eq!(worker.state, WorkerState::Typing);

        worker.state = WorkerState::Celebrating;
        assert_eq!(worker.state, WorkerState::Celebrating);

        worker.state = WorkerState::Confused;
        assert_eq!(worker.state, WorkerState::Confused);

        worker.state = WorkerState::Collaborating;
        assert_eq!(worker.state, WorkerState::Collaborating);
    }

    #[test]
    fn test_worker_specialties() {
        let config = Config::default();
        let office = Office::new(&config);

        // Check that specialties are assigned in order
        assert_eq!(office.workers[0].specialty, WorkerSpecialty::Writer);
        assert_eq!(office.workers[1].specialty, WorkerSpecialty::Researcher);
        assert_eq!(office.workers[2].specialty, WorkerSpecialty::Analyst);
        assert_eq!(office.workers[3].specialty, WorkerSpecialty::Creative);
        assert_eq!(office.workers[4].specialty, WorkerSpecialty::Editor);
    }
}
