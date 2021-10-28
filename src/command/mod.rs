use crate::settings::CONFIG_PATH;
use sdl2::keyboard::Keycode;
use std::fs;
use std::path::Path;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    str::FromStr,
};
use strum_macros::{Display, EnumString};
use xmltree;

// a command file is put into the user's config directory upon first run
// <command name="ToolPen" key = "A">
#[derive(Copy, Clone, EnumString, Display, Debug, PartialEq)]
pub enum Command {
    // zoom
    ResetScale,
    ZoomIn,
    ZoomOut,

    // move camera
    NudgeUp,
    NudgeDown,
    NudgeLeft,
    NudgeRight,

    // tools
    ToolPan,
    ToolPen,
    ToolSelect,
    ToolZoom,
    ToolVWS,
    ToolMeasure,
    ToolAnchors,
    ToolShapes,

    // selection
    DeleteSelection,
    SelectAll,
    CopySelection,
    PasteSelection,
    CutSelection,

    // history
    HistoryUndo,
    HistoryRedo,

    // I/O
    IOOpen,
    IOSave,
    IOSaveAs,
    IOSaveFlatten,
    IOFlatten,
    IOExport,

    // view modes
    TogglePointLabels,
    TogglePreviewMode,

    // console
    ToggleConsole,

    // misc
    Quit,
    ReverseContour,

    // debug
    SkiaDump,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CommandMod {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl CommandMod {
    pub const fn none() -> Self {
        Self {
            shift: false,
            ctrl: false,
            alt: false,
        }
    }
}

impl Default for CommandMod {
    fn default() -> Self {
        Self::none()
    }
}

use std::convert::TryFrom;
impl TryFrom<&str> for CommandMod {
    type Error = ();
    fn try_from(s: &str) -> Result<CommandMod, ()> {
        match s {
            "CtrlMod" => Ok(CommandMod {
                ctrl: true,
                ..CommandMod::default()
            }),
            "CtrlAltMod" | "AltCtrlMod" => Ok(CommandMod {
                alt: true,
                ctrl: true,
                ..CommandMod::default()
            }),
            "CtrlShiftMod" | "ShiftCtrlMod" => Ok(CommandMod {
                ctrl: true,
                shift: true,
                ..CommandMod::default()
            }),
            "CtrlShiftAltMod" | "ShiftCtrlAltMod" | "AltCtrlShiftMod" | "AltShiftCtrlMod"
            | "ShiftAltCtrlMod" | "CtrlAltShiftMod" => Ok(CommandMod {
                ctrl: true,
                shift: true,
                alt: true,
                ..CommandMod::default()
            }),
            "ShiftMod" => Ok(CommandMod {
                shift: true,
                ..CommandMod::default()
            }),
            "ShiftAltMod" | "AltShiftMod" => Ok(CommandMod {
                alt: true,
                shift: true,
                ..CommandMod::default()
            }),
            "AltMod" => Ok(CommandMod {
                alt: true,
                ..CommandMod::default()
            }),
            _ => Err(()),
        }
    }
}

pub struct CommandInfo {
    pub command: Command,
    pub command_mod: CommandMod,
}

pub fn initialize_keybinds() {
    let binding_xml = load_keybinding_xml();
    let mut config =
        xmltree::Element::parse(binding_xml.as_bytes()).expect("Invalid keybinding XML!");

    let mut hm: HashMap<(Keycode, Option<CommandMod>), Command> = HashMap::new();

    while let Some(binding) = config.take_child("binding") {
        let keycode = binding
            .attributes
            .get("key")
            .expect("Binding does not have a key associated!");
        let command = binding
            .attributes
            .get("command")
            .expect("Binding does not have a command associated!");
        let modifier = binding.attributes.get("mod");

        let command_mod: Option<CommandMod> = if let Some(m) = modifier {
            CommandMod::try_from(m.as_str())
                .map(|m| Some(m))
                .unwrap_or(None)
        } else {
            None
        };

        let command_enum = Command::from_str(command).expect("Invalid command string!");
        let keycode_enum =
            sdl2::keyboard::Keycode::from_name(keycode).expect("Invalid keycode string!");

        hm.insert((keycode_enum, command_mod), command_enum);
    }

    KEYMAP.with(|v| {
        v.borrow_mut().keybindings = hm;
    })
}

pub fn keys_down_to_mod(keys_down: &HashSet<Keycode>) -> Option<CommandMod> {
    let ret = CommandMod {
        ctrl: keys_down.contains(&Keycode::LCtrl) || keys_down.contains(&Keycode::RCtrl),
        shift: keys_down.contains(&Keycode::LShift) || keys_down.contains(&Keycode::RShift),
        alt: keys_down.contains(&Keycode::LAlt) || keys_down.contains(&Keycode::RAlt),
    };
    if !ret.ctrl && !ret.shift && !ret.alt {
        None
    } else {
        Some(ret)
    }
}

pub fn keycode_to_command(keycode: &Keycode, keys_down: &HashSet<Keycode>) -> Option<CommandInfo> {
    let command_enum = KEYMAP.with(|v| {
        if let Some(key) = v
            .borrow()
            .keybindings
            .get(&(*keycode, keys_down_to_mod(keys_down)))
        {
            return Some(*key);
        }

        None
    });

    if let Some(command_enum) = command_enum {
        return Some(CommandInfo {
            command: command_enum,
            command_mod: CommandMod::none(),
        });
    }

    return None;
}

fn load_keybinding_xml() -> String {
    // check for a keybinding file in our local directory first
    let config_path = Path::new("./keybindings.xml");
    let config_string = fs::read_to_string(&config_path);

    if let Ok(config_string) = config_string {
        return config_string;
    }

    let mut pb = CONFIG_PATH.clone().to_path_buf();

    pb.push("keybindings");
    pb.set_extension("xml");

    let path = pb.as_path();
    let config_string = fs::read_to_string(path);

    if let Ok(config_string) = config_string {
        return config_string;
    }

    // We didn't find either so we're gonna return our default
    DEFAULT_KEYBINDINGS.to_owned()
}

const DEFAULT_KEYBINDINGS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/default_keymap.xml"
));

struct KeyData {
    keybindings: HashMap<(Keycode, Option<CommandMod>), Command>,
}

thread_local! {
    static KEYMAP: RefCell<KeyData> = RefCell::new(KeyData{ keybindings: HashMap::new() });
}
