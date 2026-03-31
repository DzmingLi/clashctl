use crossterm::event::{KeyCode as KC, KeyEvent as KE, KeyModifiers as KM};

use crate::KeyBinding;

fn parse_key(key: &str) -> Option<KC> {
    match key.to_lowercase().as_str() {
        "space" => Some(KC::Char(' ')),
        "esc" | "escape" => Some(KC::Esc),
        "enter" | "return" => Some(KC::Enter),
        "tab" => Some(KC::Tab),
        "backspace" => Some(KC::Backspace),
        "delete" | "del" => Some(KC::Delete),
        "insert" | "ins" => Some(KC::Insert),
        "home" => Some(KC::Home),
        "end" => Some(KC::End),
        "pageup" => Some(KC::PageUp),
        "pagedown" => Some(KC::PageDown),
        "up" => Some(KC::Up),
        "down" => Some(KC::Down),
        "left" => Some(KC::Left),
        "right" => Some(KC::Right),
        "f1" => Some(KC::F(1)),
        "f2" => Some(KC::F(2)),
        "f3" => Some(KC::F(3)),
        "f4" => Some(KC::F(4)),
        "f5" => Some(KC::F(5)),
        "f6" => Some(KC::F(6)),
        "f7" => Some(KC::F(7)),
        "f8" => Some(KC::F(8)),
        "f9" => Some(KC::F(9)),
        "f10" => Some(KC::F(10)),
        "f11" => Some(KC::F(11)),
        "f12" => Some(KC::F(12)),
        s if s.len() == 1 => Some(KC::Char(s.chars().next().unwrap())),
        _ => None,
    }
}

fn parse_modifiers(mods: &[String]) -> KM {
    let mut result = KM::NONE;
    for m in mods {
        match m.to_lowercase().as_str() {
            "ctrl" | "control" => result |= KM::CONTROL,
            "alt" => result |= KM::ALT,
            "shift" => result |= KM::SHIFT,
            _ => {}
        }
    }
    result
}

pub fn matches_any(event: &KE, bindings: &[KeyBinding]) -> bool {
    bindings.iter().any(|b| matches_binding(event, b))
}

fn matches_binding(event: &KE, binding: &KeyBinding) -> bool {
    let Some(expected_key) = parse_key(&binding.key) else {
        return false;
    };
    let expected_mods = parse_modifiers(&binding.modifiers);
    event.code == expected_key && event.modifiers == expected_mods
}

/// Extract digit from a tab_goto keybinding match.
/// Returns the digit (1-9) if the event matches any tab_goto binding.
pub fn match_tab_goto(event: &KE, bindings: &[KeyBinding]) -> Option<u8> {
    for binding in bindings {
        if matches_binding(event, binding) {
            // Extract the digit from the binding key
            let key_str: &str = &binding.key;
            if let Ok(digit) = key_str.parse::<u8>() {
                if digit >= 1 && digit <= 9 {
                    return Some(digit);
                }
            }
        }
    }
    None
}
