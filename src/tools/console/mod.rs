// Console
use crate::user_interface::Interface;
use crate::{editor::Editor, util, CONSOLE};

use clipboard::{ClipboardContext, ClipboardProvider};
use lazy_static::lazy_static;
use log::debug;

mod commands;
pub use commands::initialize_console_commands;

use sdl2::keyboard::Keycode;
use sdl2::keyboard::Mod;

// Only called if ElementState::Pressed
pub fn set_state(v: &mut Editor, i: &mut Interface, vk: Keycode, _m: Mod) {
    CONSOLE.with(|c| match vk {
        Keycode::Escape => {
            c.borrow_mut().active(false);
        }
        Keycode::Return => {
            if c.borrow().active {
                run_command(v, i, &mut c.borrow_mut());
            }
            c.borrow_mut().active(false);
        }
        _ => {}
    });
}

const CHAR_BACKSPACE: char = '\x08';

use crate::editor::RendererConsole;
impl RendererConsole {
    ///! Handle chars which will not trigger events (so, not :, Escape or Return)
    pub fn handle_ch(&mut self, ch: char) {
        util::debug_event!("Got ch: {:x}", ch as u8);
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
pub fn run_command(v: &mut Editor, i: &mut Interface, c: &mut RendererConsole) {
    lazy_static! {
        static ref COMMAND_RE: Regex = Regex::new(r"\s+").unwrap();
    }

    let cmdline: Vec<_> = COMMAND_RE.split(&c.stdin).collect();

    let (command, args) = (&cmdline[0][1..], &cmdline[1..]);

    commands::MAP.with(|m| {
        m.borrow_mut()
            .get(command)
            .map(|(_, f)| f(v, i, args.to_vec().iter().map(|s| s.to_string()).collect()))
    });

    debug!("Command requested to be run: {:?}", (command, args));
    c.stdin.clear()
}
