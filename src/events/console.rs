// Console
use crate::CONSOLE;

use clipboard::{ClipboardContext, ClipboardProvider};

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
            c.borrow_mut().active(false);
            run_command(&mut c.borrow_mut());
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

pub fn run_command(c: &mut RendererConsole) {
    debug!("Command requested to be run: {}", &c.stdin);
    c.stdin.clear()
}
