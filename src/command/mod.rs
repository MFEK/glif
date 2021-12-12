use crate::settings::CONFIG_PATH;
use sdl2::keyboard::Keycode;
use std::path::PathBuf;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    str::FromStr,
};
use std::{env, fs};

use strum_macros::{Display, EnumString};

#[derive(Copy, Clone, EnumString, Hash, Display, Debug, PartialEq, Eq)]
pub enum CommandType {
    Zoom,
    Nudge,
    ToolSelect,
    Selection,
    History,
    IO,
    ViewMode,
    ToggleConsole,
    ExecState,
    PathOp,
    Debug,
}

// a command file is put into the user's config directory upon first run
// <command name="ToolPen" key = "A">
#[derive(Copy, Clone, EnumString, Hash, Display, Debug, PartialEq, Eq)]
pub enum Command {
    // zoom
    ResetScale,
    ZoomIn,
    ZoomOut,

    // move camera
    // ↑
    NudgeUp,
    NudgeBigUp,
    NudgeTinyUp,
    // ↓
    NudgeDown,
    NudgeBigDown,
    NudgeTinyDown,
    // ←
    NudgeLeft,
    NudgeBigLeft,
    NudgeTinyLeft,
    // →
    NudgeRight,
    NudgeBigRight,
    NudgeTinyRight,

    // tools
    ToolPan,
    ToolPen,
    ToolSelect,
    ToolZoom,
    ToolDash,
    ToolPAP,
    ToolVWS,
    ToolMeasure,
    ToolAnchors,
    ToolShapes,
    ToolGuidelines,
    ToolGrid,
    ToolImages,

    // selection
    DeleteSelection,
    SelectAll,
    CopySelection,
    PasteSelection,
    PasteSelectionInPlace,
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

impl Command {
    pub fn type_(&self) -> CommandType {
        use Command::*;
        match self {
            ResetScale | ZoomIn | ZoomOut => CommandType::Zoom,
            NudgeUp | NudgeBigUp | NudgeTinyUp | NudgeDown | NudgeBigDown | NudgeTinyDown
            | NudgeLeft | NudgeBigLeft | NudgeTinyLeft | NudgeRight | NudgeBigRight
            | NudgeTinyRight => CommandType::Nudge,
            ToolPan | ToolPen | ToolSelect | ToolZoom | ToolDash | ToolPAP | ToolVWS
            | ToolMeasure | ToolAnchors | ToolShapes | ToolGuidelines | ToolImages => {
                CommandType::ToolSelect
            }
            DeleteSelection
            | SelectAll
            | CopySelection
            | PasteSelection
            | PasteSelectionInPlace
            | CutSelection => CommandType::Selection,
            HistoryUndo | HistoryRedo => CommandType::History,
            IOOpen | IOSave | IOSaveAs | IOSaveFlatten | IOFlatten | IOExport => CommandType::IO,
            ToolGrid | TogglePointLabels | TogglePreviewMode => CommandType::ViewMode,
            ToggleConsole => CommandType::ToggleConsole,
            Quit => CommandType::ExecState,
            ReverseContour => CommandType::PathOp,
            SkiaDump => CommandType::Debug,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct CommandMod {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool, // ``Windows'' key
}

impl CommandMod {
    pub fn none() -> Self {
        Self::default()
    }
}

impl From<&str> for CommandMod {
    fn from(s: &str) -> CommandMod {
        let mut cm = CommandMod::none();
        // for "CtrlShiftMod", vec![0, 4, 9]
        let mod_caps: Vec<usize> = s
            .match_indices(|c: char| c.is_uppercase())
            .map(|(i, _)| i)
            .collect();
        // for "CtrlShiftMod", vec!["Ctrl", "Shift"]
        let mod_strs: Vec<&str> = mod_caps
            .as_slice()
            .windows(2)
            .map(|sl| &s[sl[0]..sl[1]])
            .collect();
        for m in mod_strs {
            match m {
                "Ctrl" | "Control" => {
                    cm.ctrl = true;
                }
                "Shift" => {
                    cm.shift = true;
                }
                "Alt" => {
                    cm.alt = true;
                }
                "Meta" | "Super" | "Windows" | "Gui" => {
                    cm.meta = true;
                }
                _ => (),
            }
        }
        cm
    }
}

#[test]
fn command_mod_test() {
    assert_eq!(
        CommandMod::from("CtrlShiftMod"),
        CommandMod {
            ctrl: true,
            shift: true,
            ..CommandMod::default()
        }
    );
}

pub struct CommandInfo {
    pub command: Command,
    pub command_mod: CommandMod,
}

const DEFAULT_KEYBINDINGS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/resources/default_keymap.xml"
));

use lazy_static::lazy_static;
lazy_static! {
    pub static ref CONFIG_KEYBIND_PATH: PathBuf = {
        let mut pb = CONFIG_PATH.clone();

        pb.push("keybindings");
        pb.set_extension("xml");
        pb
    };
}

fn load_keybinding_xml(ignore_local: bool) -> Result<String, Box<dyn std::error::Error>> {
    Ok(if !ignore_local {
        fs::read_to_string(&*CONFIG_KEYBIND_PATH)?
    } else {
        // We didn't find either so we're gonna return our default
        DEFAULT_KEYBINDINGS.to_owned()
    })
}

fn parse_keybinds(
    mut config: xmltree::Element,
) -> Result<HashMap<(Keycode, CommandMod), Command>, String> {
    let mut hm: HashMap<(Keycode, CommandMod), Command> = HashMap::new();

    while let Some(binding) = config.take_child("binding") {
        let keycode = binding
            .attributes
            .get("key")
            .map(|k| Ok(k))
            .unwrap_or(Err("Binding does not have a key associated!"))?;
        let command = binding
            .attributes
            .get("command")
            .map(|c| Ok(c))
            .unwrap_or(Err("Binding does not have a command associated!"))?;
        let modifier = binding.attributes.get("mod");

        let command_mod = modifier.map(|m| m.as_str()).unwrap_or("").into();

        let command_enum = Command::from_str(command)
            .map(|c| Ok(c))
            .unwrap_or(Err("Invalid command string!"))?;
        let keycode_enum = sdl2::keyboard::Keycode::from_name(keycode)
            .map(|kc| Ok(kc))
            .unwrap_or(Err("Invalid keycode string!"))?;

        hm.insert((keycode_enum, command_mod), command_enum);
    }

    Ok(hm)
}

pub fn initialize_keybinds() {
    let default_xml = load_keybinding_xml(true).unwrap();
    let binding_xml = load_keybinding_xml(false);
    let config_res =
        xmltree::Element::parse(binding_xml.as_ref().unwrap_or(&default_xml).as_bytes());
    let default_res =
        xmltree::Element::parse(default_xml.as_bytes()).expect("Invalid default keybinding XML‽");
    let config = match config_res {
        Ok(el) => el,
        Err(e) => {
            log::warn!("Keybinds XML invalid ({:?}), using default", e);
            default_res.clone()
        }
    };

    let default_keybinds = parse_keybinds(default_res).unwrap();
    let mut keybinds = parse_keybinds(config).unwrap_or_else(|e| {
        log::warn!("Keybinds XML invalid ({}), using default", e);
        default_keybinds.clone()
    });
    let default_commands: HashSet<_> = default_keybinds.values().into_iter().collect();
    let commands: HashSet<_> = keybinds.values().into_iter().collect();

    if commands != default_commands {
        log::warn!(
            "Your keybinds are incomplete, missing {:?}; using defaults for all",
            &default_commands - &commands
        );
        keybinds = default_keybinds;
    }

    if env::var("NO_WRITE_DEFAULT_KEYBINDS").is_err() && binding_xml.is_err() {
        static NO_WRITE_DEFAULT_KEYBINDS: &str =
            "To disable this write set environment variable NO_WRITE_DEFAULT_KEYBINDS.";
        match fs::write(
            &*CONFIG_KEYBIND_PATH,
            DEFAULT_KEYBINDINGS.to_owned().into_bytes(),
        ) {
            Ok(_) => log::info!(
                "Wrote default keybinds to `{}`. {}",
                (&*CONFIG_KEYBIND_PATH).display(),
                NO_WRITE_DEFAULT_KEYBINDS
            ),
            Err(_) => log::warn!(
                "Could not write default keybinds to `{}`? {}",
                (&*CONFIG_KEYBIND_PATH).display(),
                NO_WRITE_DEFAULT_KEYBINDS
            ),
        }
    }

    KEYMAP.with(|v| {
        v.borrow_mut().keybindings = keybinds;
    })
}

impl CommandMod {
    pub fn from_keys_down(keys_down: &HashSet<Keycode>) -> CommandMod {
        CommandMod {
            ctrl: keys_down.contains(&Keycode::LCtrl) || keys_down.contains(&Keycode::RCtrl),
            shift: keys_down.contains(&Keycode::LShift) || keys_down.contains(&Keycode::RShift),
            alt: keys_down.contains(&Keycode::LAlt) || keys_down.contains(&Keycode::RAlt),
            meta: keys_down.contains(&Keycode::LGui) || keys_down.contains(&Keycode::RGui),
        }
    }
}

pub fn keycode_to_command(keycode: &Keycode, keys_down: &HashSet<Keycode>) -> Option<CommandInfo> {
    let command_enum = KEYMAP.with(|v| {
        if let Some(key) = v
            .borrow()
            .keybindings
            .get(&(*keycode, CommandMod::from_keys_down(keys_down)))
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

    None
}

struct KeyData {
    keybindings: HashMap<(Keycode, CommandMod), Command>,
}

thread_local! {
    static KEYMAP: RefCell<KeyData> = RefCell::new(KeyData{ keybindings: HashMap::new() });
}
