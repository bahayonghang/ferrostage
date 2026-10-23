//! Raw keybindings and diagnostics. Runtime grammar belongs to the UI.

use serde::Serialize;

/// One raw key-binding diagnostic.
///
/// These records are not a dispatch map. Task 07 parses GPUI key syntax and
/// must not run two actions for a [`BindingIssue::Conflict`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BindingIssue {
    /// An action list contains `""`.
    Empty {
        /// Action field name, such as `next`.
        action: &'static str,
    },
    /// The same string appears twice on one action.
    Duplicate {
        /// Action field name.
        action: &'static str,
        /// Truncated binding text.
        binding: String,
    },
    /// The same non-empty string is assigned to two actions.
    Conflict {
        /// Truncated binding text.
        binding: String,
        /// The two action field names, in schema order.
        actions: [&'static str; 2],
    },
}

/// `keybindings` section. Values are unparsed strings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Keybindings {
    /// Advance to the next slide.
    pub next: Vec<String>,
    /// Return to the previous slide.
    pub previous: Vec<String>,
    /// Start the presentation.
    pub start_presentation: Vec<String>,
    /// Stop the presentation.
    pub stop_presentation: Vec<String>,
    /// Blank the audience to black.
    pub blank_black: Vec<String>,
    /// Blank the audience to white.
    pub blank_white: Vec<String>,
    /// Exchange presenter and audience roles.
    pub swap_screens: Vec<String>,
    /// Toggle the laser pointer.
    pub toggle_laser: Vec<String>,
    /// Toggle the pen.
    pub toggle_pen: Vec<String>,
    /// Jump to a slide.
    pub jump: Vec<String>,
}

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            // Presenter remotes such as Deli clickers send Up/Down arrows.
            next: owned(&["PageDown", "Space", "Right", "Down"]),
            previous: owned(&["PageUp", "Backspace", "Left", "Up"]),
            start_presentation: owned(&["F5"]),
            stop_presentation: owned(&["Escape"]),
            blank_black: owned(&["B"]),
            blank_white: owned(&["W"]),
            swap_screens: owned(&["S"]),
            toggle_laser: owned(&["L"]),
            toggle_pen: owned(&["P"]),
            jump: owned(&["G"]),
        }
    }
}

impl Keybindings {
    /// Action name and binding list, in file order.
    #[must_use]
    pub fn actions(&self) -> [(&'static str, &[String]); 10] {
        [
            ("next", &self.next),
            ("previous", &self.previous),
            ("start_presentation", &self.start_presentation),
            ("stop_presentation", &self.stop_presentation),
            ("blank_black", &self.blank_black),
            ("blank_white", &self.blank_white),
            ("swap_screens", &self.swap_screens),
            ("toggle_laser", &self.toggle_laser),
            ("toggle_pen", &self.toggle_pen),
            ("jump", &self.jump),
        ]
    }

    /// Replace untouched legacy `next`/`previous` defaults with the current
    /// ones, which add the arrow keys sent by presenter remotes.
    ///
    /// Customized lists are kept, and an arrow already bound to another action
    /// is not taken from it.
    pub(crate) fn upgrade_legacy_defaults(&mut self) {
        let current = Self::default();
        if self.next == owned(LEGACY_NEXT) && self.unclaimed("Down", "next") {
            self.next = current.next;
        }
        if self.previous == owned(LEGACY_PREVIOUS) && self.unclaimed("Up", "previous") {
            self.previous = current.previous;
        }
    }

    /// No action other than `owner` lists `key` (ASCII case-insensitive).
    fn unclaimed(&self, key: &str, owner: &str) -> bool {
        self.actions().iter().all(|(action, values)| {
            *action == owner
                || !values
                    .iter()
                    .any(|value| value.trim().eq_ignore_ascii_case(key))
        })
    }

    /// Duplicates, empty strings, and cross-action conflicts.
    ///
    /// The binding lists themselves are left unchanged.
    #[must_use]
    pub fn issues(&self) -> Vec<BindingIssue> {
        let mut issues = Vec::new();
        let actions = self.actions();
        for (action, values) in actions {
            let mut seen: Vec<&str> = Vec::new();
            for value in values {
                if value.is_empty() {
                    issues.push(BindingIssue::Empty { action });
                }
                if seen.contains(&value.as_str()) {
                    issues.push(BindingIssue::Duplicate {
                        action,
                        binding: brief_binding(value),
                    });
                } else {
                    seen.push(value.as_str());
                }
            }
        }
        for (index, (action, values)) in actions.iter().enumerate() {
            for value in unique_nonempty(values) {
                for (other, other_values) in actions.iter().skip(index + 1) {
                    if other_values.iter().any(|candidate| candidate == value) {
                        issues.push(BindingIssue::Conflict {
                            binding: brief_binding(value),
                            actions: [action, other],
                        });
                    }
                }
            }
        }
        issues
    }
}

/// `next` default before arrow keys were added.
const LEGACY_NEXT: &[&str] = &["PageDown", "Space", "Right"];
/// `previous` default before arrow keys were added.
const LEGACY_PREVIOUS: &[&str] = &["PageUp", "Backspace", "Left"];

fn owned(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn unique_nonempty(values: &[String]) -> Vec<&str> {
    let mut unique = Vec::new();
    for value in values {
        if value.is_empty() || unique.contains(&value.as_str()) {
            continue;
        }
        unique.push(value.as_str());
    }
    unique
}

fn brief_binding(value: &str) -> String {
    const MAX: usize = 64;
    if value.chars().count() <= MAX {
        return value.to_owned();
    }
    let mut brief: String = value.chars().take(MAX).collect();
    brief.push('…');
    brief
}
