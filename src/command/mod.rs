use sdl2::{keyboard::Keycode};
use strum_macros::{Display, EnumString};
use std::{cell::RefCell, collections::{HashMap, HashSet}, str::FromStr};
use xmltree;
use app_dirs::*;
use std::path::Path;
use std::fs::read_to_string;

// a command file is put into the user's config directory upon first run
// <command name="ToolPen" key = "A">
// <mod name= "Shift" key = "shift">
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
    ToolPen,
    ToolSelect,
    ToolZoom,
    ToolVWS,

    // view modes
    TogglePointLabels,
    TogglePreviewMode,

    ShiftMod,
    CtrlMod
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CommandMod {
    pub shift: bool,
    pub ctrl: bool
}

pub struct CommandInfo {
    pub command: Command,
    pub command_mod: CommandMod
}

pub fn initialize_keybinds()
{
    let binding_xml = load_keybinding_xml();
    let mut config = xmltree::Element::parse(binding_xml.as_bytes()).expect("Invalid keybinding XML!");

    let mut hm = HashMap::new();
    while let Some(binding) = config.take_child("binding") {
        let keycode = binding.attributes.get("key").expect("Binding does not have a key associated!");
        let command = binding.attributes.get("command").expect("Binding does not have a command associated!");

        let command_enum = Command::from_str(command).expect("Invalid command string!");
        let keycode_enum = sdl2::keyboard::Keycode::from_name(keycode).expect("Invalid keycode string!");

        hm.insert(keycode_enum, command_enum);
    }

    KEYMAP.with(|v|{
        v.borrow_mut().keybindings = hm;
    })
}

pub fn keycode_to_command(keycode: &Keycode, keys_down: &HashSet<Keycode>) -> Option<CommandInfo>
{
    let command_enum =  KEYMAP.with(|v| {
        if let Some(key) = v.borrow().keybindings.get(keycode) {
            return Some(*key)
        }

        None
    });

    println!("{:?} {:?}", command_enum, keycode);
    if let Some(command_enum) = command_enum {
        return Some(CommandInfo {
            command: command_enum,
            command_mod: key_down_to_mod(keys_down) 
        })
    }

    return None
}

// kinda clunky but it works
pub fn key_down_to_mod(keys_down: &HashSet<Keycode>) -> CommandMod
{
    let mut keymod = CommandMod {
        shift: false,
        ctrl: false
    };

    KEYMAP.with(|v|{
        for (key, value) in v.borrow().keybindings.iter() {
            match value {
                Command::ShiftMod => {
                    if keys_down.get(&key).is_some() {
                        keymod.shift = true;
                    }
                }
                Command::CtrlMod => {
                    if keys_down.get(&key).is_some() {
                        keymod.ctrl = true;
                    }
                }
                _ => {}
            }
        }
    });

    return keymod;
}

fn load_keybinding_xml() -> String
{
    // check for a keybinding file in our local directory first
    let config_path = Path::new("./keybindings.xml");
    let config_string = read_to_string(&config_path);

    if let Ok(config_string) = config_string {
        return config_string;
    }
    
    // Next we check in an OS appropriate app directory
    let config_path = app_dir(AppDataType::UserConfig, &APP_INFO, "glif");

    if let Ok(mut pb) = config_path {
        pb.push("keybindings");
        pb.set_extension("xml");

        let path = pb.as_path();
        let config_string = read_to_string(path);

        if let Ok(config_string) = config_string {
            return config_string;
        }
    }

    // We didn't find either so we're gonna return our default
    DEFAULT_KEYBINDINGS.to_owned()
}

const APP_INFO: AppInfo = AppInfo{name:"MFEK", author: "MFEK team"};
const DEFAULT_KEYBINDINGS: &str = include_str!("default_keymap.xml");

struct KeyData {
    keybindings: HashMap<Keycode, Command>
}

thread_local! {
    static KEYMAP: RefCell<KeyData> = RefCell::new(KeyData{ keybindings: HashMap::new() });
}