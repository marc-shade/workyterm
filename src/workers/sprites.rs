//! Pixel art sprite definitions for workers

/// ASCII art sprite frames for different worker states
pub struct SpriteSet {
    pub idle: Vec<Vec<&'static str>>,
    pub thinking: Vec<Vec<&'static str>>,
    pub typing: Vec<Vec<&'static str>>,
    pub celebrating: Vec<Vec<&'static str>>,
    pub confused: Vec<Vec<&'static str>>,
    pub collaborating: Vec<Vec<&'static str>>,
}

impl Default for SpriteSet {
    fn default() -> Self {
        Self {
            idle: vec![
                vec!["  O  ", " /|\\ ", " / \\ "],
                vec!["  o  ", " /|\\ ", " / \\ "],
                vec!["  O  ", " /|\\ ", " / \\ "],
                vec!["  O  ", " /|\\", "  / \\"],
            ],
            thinking: vec![
                vec!["  O? ", " /|\\ ", " / \\ "],
                vec!["  O  ", "\\|/  ", " / \\ "],
                vec!["  O! ", " /|\\ ", " / \\ "],
                vec!["  O  ", " \\|/ ", " / \\ "],
            ],
            typing: vec![
                vec!["  O  ", " /|_ ", " / \\ "],
                vec!["  O  ", " /|\\~", " / \\ "],
                vec!["  O  ", " _|/ ", " / \\ "],
                vec!["  O  ", "~/|\\ ", " / \\ "],
            ],
            celebrating: vec![
                vec!["\\O/  ", " |   ", "/ \\  "],
                vec![" \\O/ ", "  |  ", " / \\ "],
                vec!["  \\O/", "   | ", "  / \\"],
                vec![" \\O/ ", "  |  ", " / \\ "],
            ],
            confused: vec![
                vec!["  O? ", " /|\\ ", " / \\ "],
                vec!["  ?O ", " /|\\ ", " / \\ "],
                vec!["  O?!", " /|\\ ", " / \\ "],
                vec!["  O  ", " /|?\\ ", " / \\ "],
            ],
            collaborating: vec![
                vec!["  O> ", " /|\\ ", " / \\ "],
                vec!["  O>>", " /|\\ ", " / \\ "],
                vec!["  O> ", " /|\\>", " / \\ "],
                vec!["  O  ", " /|\\>", " / \\>"],
            ],
        }
    }
}

/// Office furniture and decorations
pub struct OfficeFurniture;

impl OfficeFurniture {
    pub fn desk() -> Vec<&'static str> {
        vec![
            "╔══════════╗",
            "║ ▄▄▄  ▄▄▄ ║",
            "╚══════════╝",
        ]
    }

    pub fn computer() -> Vec<&'static str> {
        vec![
            "┌───┐",
            "│ ≡ │",
            "└─┬─┘",
            "  │  ",
        ]
    }

    pub fn plant() -> Vec<&'static str> {
        vec![
            " \\|/ ",
            "  |  ",
            " ╰─╯ ",
        ]
    }

    pub fn coffee_cup() -> Vec<&'static str> {
        vec![
            "  ~  ",
            " ╭─╮ ",
            " │█│D",
            " ╰─╯ ",
        ]
    }

    pub fn whiteboard() -> Vec<&'static str> {
        vec![
            "┌─────────────┐",
            "│ TODO:       │",
            "│ □ Research  │",
            "│ □ Write     │",
            "│ □ Review    │",
            "└─────────────┘",
        ]
    }
}

/// Special effect sprites
pub struct Effects;

impl Effects {
    pub fn sparkle(frame: u8) -> &'static str {
        match frame % 4 {
            0 => "✦",
            1 => "✧",
            2 => "⋆",
            _ => "·",
        }
    }

    pub fn thinking_dots(frame: u8) -> &'static str {
        match frame % 4 {
            0 => ".",
            1 => "..",
            2 => "...",
            _ => "",
        }
    }

    pub fn progress_chars() -> [char; 8] {
        ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█']
    }
}
