use crate::clap::syntax;
use anyhow::Result;
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
use egui_console::{ConsoleBuilder, ConsoleEvent, ConsoleWindow};
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct ConsoleDemo {
    // Example stuff:
    label: String,
    #[cfg_attr(feature = "persistence", serde(skip))]
    // This how you opt-out of serialization of a field
    value: f32,
    #[cfg_attr(feature = "persistence", serde(skip))]
    console_win: ConsoleWindow,
}

impl Default for ConsoleDemo {
    fn default() -> Self {
        // Create a nice dark theme for the terminal
        let dark_theme = egui_console::TerminalTheme {
            background: egui::Color32::from_rgb(30, 30, 30),
            foreground: egui::Color32::from_rgb(220, 220, 220),
            selection: egui::Color32::from_rgb(70, 70, 70),
            cursor: egui::Color32::from_rgb(255, 255, 255),
            error: egui::Color32::from_rgb(255, 85, 85),
            success: egui::Color32::from_rgb(80, 250, 123),
            warning: egui::Color32::from_rgb(255, 184, 108),
            info: egui::Color32::from_rgb(139, 233, 253),
            prompt: egui::Color32::from_rgb(189, 147, 249),
        };

        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            console_win: ConsoleBuilder::new()
                .prompt(">> ")
                .history_size(20)
                .tab_quote_character('\"')
                .theme(dark_theme)
                .build(),
        }
    }
}

impl ConsoleDemo {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        let mut app = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Self::default()
        };

        #[cfg(not(feature = "persistence"))]
        let mut app = Self::default();

        // Add all commands to the tab completion table
        for cmd in syntax().get_subcommands() {
            app.console_win
                .command_table_mut()
                .push(cmd.get_name().to_string());

            // Also add aliases
            for alias in cmd.get_visible_aliases() {
                app.console_win
                    .command_table_mut()
                    .push(alias.to_string());
            }
        }

        // Add welcome message
        app.console_win.write_info("Welcome to Enhanced Terminal v1.0.0\n");
        app.console_win.write("Type 'commands' for a list of available commands\n");
        app.console_win.prompt();

        app
    }
}

impl eframe::App for ConsoleDemo {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        #[cfg(feature = "persistence")]
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut console_response: ConsoleEvent = ConsoleEvent::None;
                            egui::Window::new("Enhanced Terminal")
                .default_width(800.0)
                .default_height(600.0)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.add_space(4.0); // Add some padding at the top
                    console_response = self.console_win.draw(ui);
                });
            if let ConsoleEvent::Command(command) = console_response {
                match self.dispatch(&command, ctx) {
                    Err(e) => {
                        let error_msg = if let Some(original_error) = e.downcast_ref::<clap::error::Error>() {
                            format!("{}", original_error)
                        } else if e.backtrace().status() == std::backtrace::BacktraceStatus::Captured {
                            format!("{} {}", e, e.backtrace())
                        } else {
                            format!("{}", e)
                        };

                        if !error_msg.is_empty() {
                            self.console_win.write_error(format!("{}
            ", error_msg));
                        }
                    },
                    Ok(string) => {
                        if !string.is_empty() {
                            // Check content to determine appropriate styling
                            if string.to_lowercase().contains("error") {
                                self.console_win.write_error(format!("{}
            ", string));
                            } else if string.to_lowercase().contains("success") || 
                                      string.to_lowercase().contains("enabled") {
                                self.console_win.write_success(format!("{}
            ", string));
                            } else if string.to_lowercase().contains("warning") {
                                self.console_win.write_warning(format!("{}
            ", string));
                            } else {
                                self.console_win.write(&format!("{}
            ", string));
                            }
                        }
                    }
                };
                self.console_win.prompt();
            }

            ui.horizontal(|ui| {
                if ui.button("Output").clicked() {
                    self.console_win.write_info("Regular output example\n");
                    self.console_win.prompt();
                }

                if ui.button("Success").clicked() {
                    self.console_win.write_success("Success message example\n");
                    self.console_win.prompt();
                }

                if ui.button("Warning").clicked() {
                    self.console_win.write_warning("Warning message example\n");
                    self.console_win.prompt();
                }

                if ui.button("Error").clicked() {
                    self.console_win.write_error("Error message example\n");
                    self.console_win.prompt();
                }

                if ui.button("Syntax Demo").clicked() {
                    self.console_win.write_info("Rust Code Example:\n");
                    let rust_code = "fn main() {\n    println!(\"Hello, world!\");\n    let x = 42;\n}";
                    self.console_win.write(rust_code);
                    self.console_win.write("\n");
                    self.console_win.prompt();
                }
            });
        });
    }
}
impl ConsoleDemo {
    pub fn dispatch(&mut self, line: &str, ctx: &egui::Context) -> Result<String> {
        // let args = line.split_whitespace();
        let args = shlex::split(line).ok_or(anyhow::anyhow!("cannot parse"))?;
        // parse with clap
        let matches = syntax().try_get_matches_from(args)?;
        // execute the command
        match matches.subcommand() {
            Some(("cd", args)) => {
                let dir = args.get_one::<String>("directory").unwrap();
                std::env::set_current_dir(dir)?;
                let cwd = std::env::current_dir()?;
                Ok(format!("Current working directory: {}", cwd.display()))
            }
            Some(("dark", _)) => {
                //  let ctx = egui::Context::default();
                ctx.set_visuals(egui::Visuals::dark());
                Ok("Dark mode enabled".to_string())
            }
            Some(("light", _)) => {
                //   let ctx = egui::Context::default();
                ctx.set_visuals(egui::Visuals::light());
                Ok("Light mode enabled".to_string())
            }
            Some(("quit", _)) => {
                //   let ctx = egui::Context::default();
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                Ok("Bye".to_string())
            }
            Some(("clear_screen", _)) => {
                self.console_win.clear();
                Ok("".to_string())
            }
            Some(("dir", args)) => {
                let filter = if let Some(filter) = args.get_one::<String>("filter") {
                    filter.clone()
                } else {
                    "".to_string()
                };
                let entries = std::fs::read_dir(".")?;
                let mut result = String::new();
                for entry in entries {
                    let entry = entry?;
                    let path = entry.path();
                    if path.display().to_string().contains(filter.as_str()) {
                        result.push_str(&format!("{}\n", path.display()));
                    }
                }
                Ok(result)
            }
            Some(("history", _)) => {
                let history = self.console_win.get_history();
                let mut result = String::new();
                for (i, line) in history.iter().enumerate() {
                    result.push_str(&format!("{}: {}\n", i, line));
                }
                Ok(result)
            }
            Some(("clear_history", _)) => {
                self.console_win.clear_history();
                Ok("".to_string())
            }
            Some(("cat", args)) => {
                let file_path = args.get_one::<String>("file").unwrap();
                match std::fs::read_to_string(file_path) {
                    Ok(content) => Ok(content),
                    Err(e) => Err(anyhow::anyhow!("Error reading file: {}", e)),
                }
            }
            Some(("pwd", _)) => {
                let cwd = std::env::current_dir()?;
                Ok(format!("Current directory: {}", cwd.display()))
            }
            Some(("echo", args)) => {
                let text: Vec<&String> = args.get_many::<String>("text").unwrap_or_default().collect();
                let text_strings: Vec<String> = text.iter().map(|s| s.to_string()).collect();
                Ok(text_strings.join(" "))
            }
            Some(("commands", _)) => {
                let mut result = String::from("Available commands:\n");
                for cmd in syntax().get_subcommands() {
                    let name = cmd.get_name();
                    let about = match cmd.get_about() {
                        Some(about) => about.to_string(),
                        None => String::new(),
                    };
                    result.push_str(&format!("  {} - {}\n", name, about));
                }
                Ok(result)
            }
                            Some(("codeview", args)) => {
                let language = args.get_one::<String>("language").unwrap();
                let file_path = args.get_one::<String>("file").unwrap();

                match std::fs::read_to_string(file_path) {
                    Ok(content) => {
                        // Use syntax highlighting based on the language
                        let lang = match language.to_lowercase().as_str() {
                            "rust" => egui_console::Language::Rust,
                            "python" | "py" => egui_console::Language::Python,
                            "javascript" | "js" => egui_console::Language::JavaScript,
                            "html" => egui_console::Language::HTML,
                            "css" => egui_console::Language::CSS,
                            "json" => egui_console::Language::JSON,
                            "shell" | "bash" | "sh" => egui_console::Language::Shell,
                            _ => egui_console::Language::Plaintext,
                        };

                        self.console_win.write_info(format!("File: {} ({})", file_path, language));
                        self.console_win.write("\n");
                        self.console_win.write_code(&content, lang);
                        Ok(format!("Displayed {} with syntax highlighting", file_path))
                    },
                    Err(e) => Err(anyhow::anyhow!("Error reading file: {}", e)),
                }
                            }
            Some(("theme", args)) => {
                if let Some(theme_name) = args.get_one::<String>("name") {
                    match theme_name.to_lowercase().as_str() {
                        "dark" => {
                            let dark_theme = egui_console::TerminalTheme {
                                background: egui::Color32::from_rgb(30, 30, 30),
                                foreground: egui::Color32::from_rgb(220, 220, 220),
                                selection: egui::Color32::from_rgb(70, 70, 70),
                                cursor: egui::Color32::from_rgb(255, 255, 255),
                                error: egui::Color32::from_rgb(255, 85, 85),
                                success: egui::Color32::from_rgb(80, 250, 123),
                                warning: egui::Color32::from_rgb(255, 184, 108),
                                info: egui::Color32::from_rgb(139, 233, 253),
                                prompt: egui::Color32::from_rgb(189, 147, 249),
                            };
                            self.console_win.set_theme(dark_theme);
                            Ok("Applied dark theme".to_string())
                        },
                        "light" => {
                            let light_theme = egui_console::TerminalTheme {
                                background: egui::Color32::from_rgb(240, 240, 240),
                                foreground: egui::Color32::from_rgb(30, 30, 30),
                                selection: egui::Color32::from_rgb(180, 180, 180),
                                cursor: egui::Color32::from_rgb(0, 0, 0),
                                error: egui::Color32::from_rgb(220, 50, 50),
                                success: egui::Color32::from_rgb(0, 170, 0),
                                warning: egui::Color32::from_rgb(200, 150, 0),
                                info: egui::Color32::from_rgb(0, 100, 200),
                                prompt: egui::Color32::from_rgb(100, 50, 200),
                            };
                            self.console_win.set_theme(light_theme);
                            Ok("Applied light theme".to_string())
                        },
                        "dracula" => {
                            let dracula_theme = egui_console::TerminalTheme {
                                background: egui::Color32::from_rgb(40, 42, 54),
                                foreground: egui::Color32::from_rgb(248, 248, 242),
                                selection: egui::Color32::from_rgb(68, 71, 90),
                                cursor: egui::Color32::from_rgb(248, 248, 242),
                                error: egui::Color32::from_rgb(255, 85, 85),
                                success: egui::Color32::from_rgb(80, 250, 123),
                                warning: egui::Color32::from_rgb(241, 250, 140),
                                info: egui::Color32::from_rgb(139, 233, 253),
                                prompt: egui::Color32::from_rgb(189, 147, 249),
                            };
                            self.console_win.set_theme(dracula_theme);
                            Ok("Applied dracula theme".to_string())
                        },
                        "solarized" => {
                            let solarized_theme = egui_console::TerminalTheme {
                                background: egui::Color32::from_rgb(0, 43, 54),
                                foreground: egui::Color32::from_rgb(131, 148, 150),
                                selection: egui::Color32::from_rgb(7, 54, 66),
                                cursor: egui::Color32::from_rgb(131, 148, 150),
                                error: egui::Color32::from_rgb(220, 50, 47),
                                success: egui::Color32::from_rgb(133, 153, 0),
                                warning: egui::Color32::from_rgb(181, 137, 0),
                                info: egui::Color32::from_rgb(38, 139, 210),
                                prompt: egui::Color32::from_rgb(108, 113, 196),
                            };
                            self.console_win.set_theme(solarized_theme);
                            Ok("Applied solarized theme".to_string())
                        },
                        "nord" => {
                            let nord_theme = egui_console::TerminalTheme {
                                background: egui::Color32::from_rgb(46, 52, 64),
                                foreground: egui::Color32::from_rgb(216, 222, 233),
                                selection: egui::Color32::from_rgb(67, 76, 94),
                                cursor: egui::Color32::from_rgb(216, 222, 233),
                                error: egui::Color32::from_rgb(191, 97, 106),
                                success: egui::Color32::from_rgb(163, 190, 140),
                                warning: egui::Color32::from_rgb(235, 203, 139),
                                info: egui::Color32::from_rgb(129, 161, 193),
                                prompt: egui::Color32::from_rgb(180, 142, 173),
                            };
                            self.console_win.set_theme(nord_theme);
                            Ok("Applied nord theme".to_string())
                        },
                        _ => Ok(format!("Unknown theme: {}. Available themes: dark, light, dracula, solarized, nord", theme_name))
                    }
                } else {
                    Ok("Available themes: dark, light, dracula, solarized, nord".to_string())
                }
            }
            _ => Ok("Unknown command. Type 'help' for available commands.".to_string()),
        }
    }
}
