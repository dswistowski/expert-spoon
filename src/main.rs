use global_hotkey::GlobalHotKeyEvent;
use global_hotkey::{hotkey::HotKey, GlobalHotKeyManager};
use home::home_dir;
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use std::{env, fs};

use serde_yaml;

use tao::menu::ContextMenu;
use tao::system_tray::{SystemTrayBuilder};
use tao::{
    event::{Event},
    event_loop::{ControlFlow, EventLoop},
    menu::{ MenuItemAttributes, MenuType},
    TrayId,
};

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
enum Action {
    #[serde(rename = "open")]
    ActionOpen { command: String, args: Vec<String> },
}

#[derive(Deserialize, Debug)]
struct HotkeyConfig {
    key: String,
    name: String,
    action: Action,
}

#[derive(Deserialize, Debug)]
struct Config {
    version: u8,
    hotkeys: Vec<HotkeyConfig>,
}

fn default_path() -> Option<PathBuf> {
    match home_dir() {
        Some(dir) => Some(dir.join(".expert-spoon.yaml")),
        None => None,
    }
}

trait ExecutableConfig {
    fn execute(&self);
}

impl ExecutableConfig for Action {
    fn execute(&self) {
        match self {
            Action::ActionOpen { command, args } => {
                Command::new(command)
                    .args(args.iter())
                    .output()
                    .expect("Failed to run command}");
            }
        }
    }
}

fn env_path() -> Option<PathBuf> {
    match env::var("EXPERT_SPOON_CONFIG") {
        Ok(path) => {
            let mut buf = PathBuf::new();
            buf.push(path);
            Some(buf)
        }
        Err(_) => None,
    }
}

fn find_config<I>(paths: I) -> Option<PathBuf>
where
    I: IntoIterator<Item = PathBuf>,
{
    if let Some(result) = paths.into_iter().find(|path| path.as_path().exists()) {
        let mut return_value = PathBuf::new();
        return_value.push::<PathBuf>(result.into());
        return Some(return_value);
    }
    None
}

fn main() {
    let config = load_config();
    if config.version != 1 {
        panic!("Wrong config version, please use: 1")
    }

    let manager = GlobalHotKeyManager::new().unwrap();

    let mut tray_menu = ContextMenu::new();
    let registered_hotkeys: Vec<_> = config
        .hotkeys
        .iter()
        .map(|config| {
            let hotkey = HotKey::from_str(config.key.as_str()).expect("Cannot parse hotkey string");
            manager.register(hotkey).expect("Failed to register hotkey");
            let tray_id= TrayId::new(config.name.as_str());

            tray_menu.add_item(MenuItemAttributes::new(
                format!("{} ({})", config.name, config.key).as_str(),
            ));
            (hotkey, config.action.clone(), tray_id)
        })
        .collect();

    let event_loop = EventLoop::new();
    let global_hotkey_channel = GlobalHotKeyEvent::receiver();

    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/icon.png");

    let icon = load_icon(std::path::Path::new(path));


    let main_tray_id = TrayId::new("main-tray");
    let quit = tray_menu.add_item(MenuItemAttributes::new("Quit"));

    let _system_tray = SystemTrayBuilder::new(icon.clone(), Some(tray_menu))
        .with_id(main_tray_id)
        .with_tooltip("expert-spoon global hotkeys")
        .build(&event_loop)
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::MenuEvent {
                menu_id,
                origin: MenuType::ContextMenu,
                ..
            } => {
                if menu_id == quit.clone().id() {
                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::TrayEvent {
                id,
                ..
            } => {

                for (_, config, tray_id) in registered_hotkeys.iter() {
                    if id == tray_id.clone() {
                        config.execute();
                        break
                    }
                };
            }
            _ => (),
        }

        if let Ok(global_hotkey_event) = global_hotkey_channel.try_recv() {
            for (hotkey, config, ..) in registered_hotkeys.iter() {
                println!("{hotkey:?}");
                if global_hotkey_event.id == hotkey.id() {
                    println!("{config:?}");
                    config.execute()
                }
            }
        };
    })
}

fn load_icon(path: &std::path::Path) -> tao::system_tray::Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    tao::system_tray::Icon::from_rgba(icon_rgba, icon_width, icon_height)
        .expect("Failed to open icon")
}

fn load_config() -> Config {
    let config_path = match find_config(vec![
        default_path().unwrap(),
        env_path().unwrap_or(PathBuf::new()),
    ]) {
        None => panic!("Cannot find config in $HOME/.expert-spoon.yaml"),
        Some(path) => path,
    };

    println!("Using config: {config_path:?}");
    let config_content = fs::read_to_string(config_path).expect("Failed to read config file");
    let config: Config = serde_yaml::from_str(config_content.as_str())
        .unwrap_or_else(|error| panic!("Failed to process config file: {}", error));
    config
}
