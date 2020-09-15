// Console
use crate::CONSOLE;

use clipboard::{ClipboardContext, ClipboardProvider};

mod commands;

use winit::event::{ModifiersState, VirtualKeyCode};
// Only called if ElementState::Pressed
pub fn set_state(vk: VirtualKeyCode, m: ModifiersState) {
    CONSOLE.with(|c| match vk {
        VirtualKeyCode::Semicolon => {
            if !m.shift() {
                return;
            }
            c.borrow_mut().active(true);
        }
        VirtualKeyCode::Escape => {
            c.borrow_mut().active(false);
        }
        VirtualKeyCode::Return => {
            if c.borrow().active {
                run_command(&mut c.borrow_mut());
            }
            c.borrow_mut().active(false);
        }
        _ => {}
    });
}

const CHAR_BACKSPACE: char = '\x08';

use state::RendererConsole;
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

use regex::{self, Regex};
pub fn run_command(c: &mut RendererConsole) {
    lazy_static! {
        static ref COMMAND_RE: Regex = Regex::new(r"\s+").unwrap();
    }

    let cmdline: Vec<_> = COMMAND_RE.split(&c.stdin).collect();

    let (command, args) = (&cmdline[0][1..], &cmdline[1..]);

    commands::MAP.with(|m| {
        m.get(command)
            .map(|f| f(args.to_vec().iter().map(|s| s.to_string()).collect()))
    });

    debug!("Command requested to be run: {:?}", (command, args));
    c.stdin.clear()
}
