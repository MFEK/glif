use super::{events::*, Editor};
use crate::user_interface::{InputPrompt, Interface};

use std::ffi::OsStr;
use std::rc::Rc;
use std::sync::mpsc::TryRecvError;

use mfek_ipc::InUfo as _;

fn oss(s: &'static str) -> &'static OsStr {
    OsStr::new(s)
}

impl Editor {
    pub fn handle_filesystem_events(&mut self, i: &mut Interface) {
        'filesystem_watch: loop {
            let event = self.filesystem_watch_rx.try_recv();
            match event {
                Ok(p) => {
                    if p.file_name() == Some(oss("fontinfo.plist")) {
                        self.initialize();
                        self.dispatch_editor_event(
                            i,
                            EditorEvent::IOEvent {
                                event_type: IOEventType::FontinfoReloaded,
                                path: p,
                            },
                        );
                        log::info!("Reloaded UFO-sourced metadata, fontinfo.plist changed");
                    } else if p.extension() == Some(oss("glif"))
                        || p.extension() == Some(oss("glifjson"))
                    {
                        let filename = self.filename_or_panic();
                        if filename.file_name().unwrap() == p.file_name().unwrap() {
                            if !self.just_saved() {
                                i.push_prompt(InputPrompt::YesNo {
                                    question: "Another program/MFEKglif instance rewrote the current \nglyph. Reload? Any changes made will be lost.\n ".to_string(),
                                    afterword: "".to_string(),
                                    func: Rc::new(move |v, i, reload| {
                                        if !reload { return }
                                        v.begin_modification("Reloaded glyph due to write by another program or instance.");
                                        let filename = v.filename_or_panic();
                                        v.load_glif(i, filename);
                                        v.end_modification();
                                    }),
                                });
                                log::warn!("Another program changed this glyph!");
                            } else {
                                log::debug!("Got filesystem event from our own recent write");
                            }
                        } else {
                            let ufo_or_dir = if p.ufo().is_some() {
                                "UFO"
                            } else {
                                "directory"
                            };
                            log::info!(
                                "Another glif in this {} changed: {:?}",
                                ufo_or_dir,
                                p.file_name().unwrap()
                            );
                        }
                    } else {
                        log::debug!("Ignored write of file {:?}", p)
                    }
                }
                Err(TryRecvError::Empty) => break 'filesystem_watch,
                Err(_) => panic!("Filesystem watcher disconnected!"),
            }
        }
    }
}
