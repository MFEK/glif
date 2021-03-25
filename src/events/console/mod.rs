// Console
use crate::CONSOLE;

use clipboard::{ClipboardContext, ClipboardProvider};

mod commands;

use sdl2::keyboard::Keycode;
use sdl2::keyboard::Mod;

// Only called if ElementState::Pressed
pub fn set_state(vk: Keycode, m: Mod) {
    CONSOLE.with(|c| match vk {
        Keycode::Semicolon => {
            if !m.contains(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
                return;
            }
            c.borrow_mut().active(true);
        }
        Keycode::Escape => {
            c.borrow_mut().active(false);
        }
        Keycode::Return => {
            if c.borrow().active {
                run_command(&mut c.borrow_mut());
            }
            c.borrow_mut().active(false);
        }
        _ => {}
    });
}

const CHAR_BACKSPACE: char = '\x08';

use crate::state::RendererConsole;
impl RendererConsole {
    ///! Handle chars which will not trigger events (so, not :, Escape or Return)
    pub fn handle_ch(&mut self, ch: char) {
        debug_event!("Got ch: {:x}", ch as u8);
        if ch != CHAR_BACKSPACE {
            self.stdin.push(ch);
        } else {
            if self.stdin.len() > 1 {
                // don't delete `:`
                self.stdin.pop();
            }
        }
    }

    ///! String from clipboard
    pub fn handle_clipboard(&mut self) {
        let mut ctx: ClipboardContext =
            ClipboardProvider::new().expect("Failed to set up clipboard provider");
        if let Ok(s) = ctx.get_contents() {
            self.stdin.push_str(&s);
        }
    }
}

use regex::Regex;
pub fn run_command(c: &mut RendererConsole) {
    lazy_static! {
        static ref COMMAND_RE: Regex = Regex::new(r"\s+").unwrap();
    }

    let cmdline: Vec<_> = COMMAND_RE.split(&c.stdin).collect();

    let (command, args) = (&cmdline[0][1..], &cmdline[1..]);

    commands::MAP.with(|m| {
        m.get(command)
            .map(|(_, f)| f(args.to_vec().iter().map(|s| s.to_string()).collect()))
    });

    debug!("Command requested to be run: {:?}", (command, args));
    c.stdin.clear()
}
