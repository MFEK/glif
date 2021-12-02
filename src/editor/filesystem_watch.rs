use super::Editor;
use crate::user_interface::{InputPrompt, Interface};

use std::ffi::OsStr;
use std::rc::Rc;
use std::sync::mpsc::TryRecvError;

fn oss(s: &'static str) -> &'static OsStr {
    OsStr::new(s)
}

impl Editor {
    pub fn handle_filesystem_events(&mut self, i: &mut Interface) {
        loop {
            let event = self.filesystem_watch_rx.try_recv();
            match event {
                Ok(p) => {
                    if p.file_name() == Some(oss("fontinfo.plist")) {
                        self.initialize();
                        log::info!("Reloaded glyph, fontinfo.plist changed");
                    } else if p.extension() == Some(oss("glif"))
                        || p.extension() == Some(oss("glifjson"))
                    {
                        let filename = self.filename_or_panic();
                        if filename.file_name().unwrap() == p.file_name().unwrap() {
                            let our_change = self.history.undo_stack.last().map(|undo| {
                                undo.description == "Saved glyph"
                                    || undo.description == "Flattened glyph"
                                    || undo.description == "Exported glyph"
                            });
                            if !our_change.unwrap_or(false) {
                                i.push_prompt(InputPrompt::YesNo {
                                    question: "Another program/MFEKglif instance rewrote the current \nglyph. Reload?\n ".to_string(),
                                    func: Rc::new(move |v, reload| {
                                        if !reload { return }
                                        v.begin_modification("Reloaded glyph due to write by another program or instance.");
                                        let filename = v.glyph.as_ref().unwrap().filename.as_ref().unwrap().to_owned();
                                        v.load_glif_headless(filename);
                                        v.end_modification();
                                    }),
                                });
                                log::warn!("Another program changed this glyph!");
                            } else {
                                log::debug!("Got filesystem event from our own recent write");
                            }
                            if let Some(true) = our_change {
                                self.history.undo_stack.pop();
                            }
                        } else {
                            log::info!(
                                "Another glif in this UFO changed: {:?}",
                                p.file_name().unwrap()
                            );
                        }
                    } else {
                        log::debug!("Ignored write of file {:?} in UFO", p)
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(_) => panic!("Filesystem watcher disconnected!"),
            }
        }
    }
}
