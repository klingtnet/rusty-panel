extern crate gdk;
extern crate gio;
extern crate gtk;

#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

extern crate failure;
extern crate getopts;

extern crate shellexpand;

use gdk::prelude::*;
use gio::prelude::*;
use gtk::prelude::*;

use std::env::args;
use std::fs::File;
use std::path::Path;
use std::process::Command;

// resized to height required to show text
// autoresize does not work when window type hint is set to "dock"
const HEIGHT: i32 = 18;

const DEFAULT_PATH: &str = "~/.config/rusty-panel.yaml";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Config {
    cmd: String,
    hide_delay_ms: u64,
    timeout_s: u32,
}
impl Default for Config {
    fn default() -> Self {
        Config {
            cmd: "date".to_string(),
            hide_delay_ms: 500,
            timeout_s: 1,
        }
    }
}

fn primary_monitor() -> Option<gdk::Monitor> {
    let display = gdk::Display::get_default().expect("Failed to get default display");
    for i in 0..display.get_n_monitors() {
        if let Some(monitor) = display.get_monitor(i) {
            if monitor.is_primary() {
                return Some(monitor);
            }
        }
    }
    None
}

fn build_ui(application: &gtk::Application, config: &Config) {
    let window = gtk::ApplicationWindow::new(application);
    window.set_title("rusty-panel");

    let monitor = primary_monitor().expect("Could not determine primary monitor");
    let geometry = monitor.get_geometry();
    window.set_default_size(geometry.width, HEIGHT);

    // Do not show in application switcher.
    // The reason to hide it is that there is no reason to focus the panel since there is no user input and it is also annyoing to skip it always when using the application switcher (Alt+Tab).
    window.set_skip_pager_hint(true);
    window.set_skip_taskbar_hint(true);

    // dock specific settings
    window.set_type_hint(gdk::WindowTypeHint::Dock);
    window.set_keep_above(true);
    window.stick();

    // remove decoration
    window.set_resizable(false);
    window.set_decorated(false);

    window.set_position(gtk::WindowPosition::Center);

    // TODO: check if this is the default
    window.connect_delete_event(move |win, _| {
        win.destroy();
        Inhibit(false)
    });

    let text_view = gtk::TextView::new();
    text_view.set_editable(false);
    text_view.set_monospace(true);
    text_view.set_cursor_visible(false);
    text_view.set_justification(gtk::Justification::Center);

    window.add(&text_view);
    window.move_(0, geometry.height - HEIGHT);

    window.show_all();

    window.connect_enter_notify_event({
        let height = geometry.height;
        move |w, _event| {
            w.move_(0, height - HEIGHT);
            gtk::Inhibit(false)
        }
    });
    window.connect_leave_notify_event({
        let height = geometry.height;
        let hide_delay = config.hide_delay_ms;
        move |w, _event| {
            std::thread::sleep(std::time::Duration::from_millis(hide_delay));
            w.move_(0, height - 1);
            gtk::Inhibit(false)
        }
    });

    let expanded_cmd: String = shellexpand::tilde(&config.cmd).into();
    println!("Trying to run '{}'", expanded_cmd);
    let mut cmd = Command::new(expanded_cmd);
    gtk::timeout_add_seconds(config.timeout_s, {
        move || {
            let cmd_result = cmd.output().expect("failed to execute process");

            let output =
                String::from_utf8(cmd_result.stdout).expect("Command output was not valid UTF-8");

            text_view
                .get_buffer()
                .expect("Failed to get text-buffer")
                .set_text(&output.trim());

            gtk::Continue(true)
        }
    });
}

struct CLI {
    matches: getopts::Matches,
    options: getopts::Options,
}

fn parse_args(args: Vec<String>) -> Result<CLI, getopts::Fail> {
    let mut options = getopts::Options::new();
    options.optopt(
        "c",
        "conf",
        "path to configuration file",
        "/path/to/config.yaml",
    );
    options.optflag("h", "help", "print this help menu");
    let matches = options.parse(&args[1..])?;
    Ok(CLI { matches, options })
}

fn load_default_config() -> Result<Config, failure::Error> {
    let expanded_default_path: String = shellexpand::tilde(DEFAULT_PATH).into();
    if Path::new(&expanded_default_path).exists() {
        let file = File::open(&expanded_default_path)?;
        serde_yaml::from_reader(file).map_err(|err| err.into())
    } else {
        let file = File::create(&expanded_default_path)?;
        let default_config = Config::default();
        serde_yaml::to_writer(file, &default_config)?;
        println!("Default configuration written to '{}'", DEFAULT_PATH);
        Ok(default_config)
    }
}

fn load_config(config_path: Option<String>) -> Result<Config, failure::Error> {
    if let Some(path) = config_path {
        let expanded_path: String = shellexpand::tilde(&path).into();
        let file = File::open(&expanded_path)?;
        serde_yaml::from_reader(file).map_err(|err| err.into())
    } else {
        load_default_config()
    }
}

fn main() -> Result<(), failure::Error> {
    let args: Vec<String> = args().collect();
    let program = args[0].clone();
    let cli = parse_args(args)?;
    if cli.matches.opt_present("h") {
        let usage_msg = cli.options.usage(&format!("Usage: {} [options]", program));
        println!("{}", usage_msg);
        return Ok(());
    }
    let application = gtk::Application::new(
        "com.github.klingtnet.shellpanel",
        gio::ApplicationFlags::empty(),
    )?;

    let config = load_config(cli.matches.opt_str("c"))?;
    application.connect_startup(move |app| {
        build_ui(app, &config);
    });

    application.connect_activate(|_| {});
    application.run(&Vec::new());
    Ok(())
}
