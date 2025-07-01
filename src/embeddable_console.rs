use crate::{ConsoleBuilder, ConsoleEvent, ConsoleWindow, TerminalTheme};

/// A simple embeddable console that can be integrated into any egui application
pub struct EmbeddableConsole {
    console: ConsoleWindow,
    is_visible: bool,
    error_message: Option<String>,
}

impl Default for EmbeddableConsole {
    fn default() -> Self {
        Self::new()
    }
}

impl EmbeddableConsole {
    /// Create a new embeddable console with default settings
    pub fn new() -> Self {
        let dark_theme = TerminalTheme {
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

        let mut console = ConsoleBuilder::new()
            .prompt(">> ")
            .history_size(100)
            .theme(dark_theme)
            .build();

        // Initialize koto runtime
        if let Err(e) = console.enable_koto() {
            eprintln!("Failed to initialize Koto runtime: {}", e);
        }

        // Add welcome message
        console.write_info("Embedded Console with Koto Scripting\n");
        console.write("Type commands or Koto scripts. Use 'koto>' prefix for explicit Koto scripts.\n");
        console.write("Type 'help' for available commands.\n");
        console.prompt();

        Self {
            console,
            is_visible: true,
            error_message: None,
        }
    }

    /// Create a new embeddable console with custom settings
    pub fn with_settings(prompt: &str, theme: TerminalTheme, enable_koto: bool) -> Self {
        let mut console = ConsoleBuilder::new()
            .prompt(prompt)
            .theme(theme)
            .build();

        if enable_koto {
            if let Err(e) = console.enable_koto() {
                eprintln!("Failed to initialize Koto runtime: {}", e);
            }
        }

        console.prompt();

        Self {
            console,
            is_visible: true,
            error_message: None,
        }
    }

    /// Show/hide the console
    pub fn set_visible(&mut self, visible: bool) {
        self.is_visible = visible;
    }

    /// Check if console is visible
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    /// Toggle console visibility
    pub fn toggle_visibility(&mut self) {
        self.is_visible = !self.is_visible;
    }

    /// Draw the console as a window
    pub fn draw_window(&mut self, ctx: &egui::Context) -> ConsoleEvent {
        if !self.is_visible {
            return ConsoleEvent::None;
        }

        let mut event = ConsoleEvent::None;
        
        egui::Window::new("Console")
            .default_width(800.0)
            .default_height(400.0)
            .resizable(true)
            .show(ctx, |ui| {
                event = self.draw_inline(ui);
            });

        event
    }

    /// Draw the console inline in the current UI
    pub fn draw_inline(&mut self, ui: &mut egui::Ui) -> ConsoleEvent {
        // Show error message if any
        if let Some(ref error_msg) = self.error_message.clone() {
            ui.colored_label(egui::Color32::RED, format!("Error: {}", error_msg));
            self.error_message = None;
        }

        let event = self.console.draw(ui);

        // Handle console events
        match &event {
            ConsoleEvent::Command(command) => {
                self.handle_builtin_command(command)
            }
            ConsoleEvent::KotoScript(script) => {
                self.handle_koto_script(script)
            }
            ConsoleEvent::None => {}
        }

        event
    }

    /// Draw the console as a bottom panel
    pub fn draw_bottom_panel(&mut self, ctx: &egui::Context) -> ConsoleEvent {
        if !self.is_visible {
            return ConsoleEvent::None;
        }

        let mut event = ConsoleEvent::None;

        egui::TopBottomPanel::bottom("console_panel")
            .default_height(200.0)
            .resizable(true)
            .show(ctx, |ui| {
                event = self.draw_inline(ui);
            });

        event
    }

    /// Handle built-in commands
    fn handle_builtin_command(&mut self, command: &str) {
        let parts: Vec<&str> = command.trim().split_whitespace().collect();
        if parts.is_empty() {
            self.console.prompt();
            return;
        }

        match parts[0] {
            "help" => {
                self.console.write_info("Available commands:\n");
                self.console.write("  help          - Show this help message\n");
                self.console.write("  clear         - Clear the console\n");
                self.console.write("  history       - Show command history\n");
                self.console.write("  koto_mode on  - Enable Koto scripting mode\n");
                self.console.write("  koto_mode off - Disable Koto scripting mode\n");
                self.console.write("  koto> <script> - Execute Koto script\n");
                self.console.write("  theme <name>   - Change theme (dark, light, dracula)\n");
                self.console.write("\nKoto scripting functions:\n");
                self.console.write("  print(...)         - Print text\n");
                self.console.write("  console.info(...)  - Print info message\n");
                self.console.write("  console.success(...) - Print success message\n");
                self.console.write("  console.warning(...) - Print warning message\n");
                self.console.write("  console.error(...)   - Print error message\n");
                self.console.write("  set_theme(bg, fg)    - Set theme colors\n");
                self.console.write("  clear_console()      - Clear console\n");
            }
            "clear" => {
                self.console.clear();
            }
            "history" => {
                let history = self.console.get_history();
                for (i, cmd) in history.iter().enumerate() {
                    self.console.write(&format!("{}: {}\n", i + 1, cmd));
                }
            }
            "koto_mode" => {
                if parts.len() > 1 {
                    match parts[1] {
                        "on" => {
                            if let Err(e) = self.console.enable_koto() {
                                self.error_message = Some(format!("Failed to enable Koto mode: {}", e));
                            } else {
                                self.console.write_success("Koto scripting mode enabled\n");
                            }
                        }
                        "off" => {
                            self.console.disable_koto();
                            self.console.write_info("Koto scripting mode disabled\n");
                        }
                        _ => {
                            self.console.write_error("Usage: koto_mode [on|off]\n");
                        }
                    }
                } else {
                    let mode = if self.console.is_koto_mode() { "on" } else { "off" };
                    self.console.write(&format!("Koto mode is currently: {}\n", mode));
                }
            }
            "theme" => {
                if parts.len() > 1 {
                    self.set_theme(parts[1]);
                } else {
                    self.console.write("Available themes: dark, light, dracula\n");
                }
            }
            _ => {
                self.console.write_error(&format!("Unknown command: {}. Type 'help' for available commands.\n", parts[0]));
            }
        }
        self.console.prompt();
    }

    /// Handle koto script execution
    fn handle_koto_script(&mut self, script: &str) {
        if script.trim().is_empty() {
            self.console.prompt();
            return;
        }

        match self.console.execute_koto(script) {
            Ok(result) => {
                if !result.trim().is_empty() {
                    self.console.write(&result);
                }
            }
            Err(e) => {
                self.console.write_error(&format!("Koto error: {}\n", e));
            }
        }
        self.console.prompt();
    }

    /// Set console theme
    fn set_theme(&mut self, theme_name: &str) {
        let theme = match theme_name {
            "dark" => TerminalTheme {
                background: egui::Color32::from_rgb(30, 30, 30),
                foreground: egui::Color32::from_rgb(220, 220, 220),
                selection: egui::Color32::from_rgb(70, 70, 70),
                cursor: egui::Color32::from_rgb(255, 255, 255),
                error: egui::Color32::from_rgb(255, 85, 85),
                success: egui::Color32::from_rgb(80, 250, 123),
                warning: egui::Color32::from_rgb(255, 184, 108),
                info: egui::Color32::from_rgb(139, 233, 253),
                prompt: egui::Color32::from_rgb(189, 147, 249),
            },
            "light" => TerminalTheme {
                background: egui::Color32::from_rgb(240, 240, 240),
                foreground: egui::Color32::from_rgb(30, 30, 30),
                selection: egui::Color32::from_rgb(180, 180, 180),
                cursor: egui::Color32::from_rgb(0, 0, 0),
                error: egui::Color32::from_rgb(220, 50, 50),
                success: egui::Color32::from_rgb(0, 170, 0),
                warning: egui::Color32::from_rgb(200, 150, 0),
                info: egui::Color32::from_rgb(0, 100, 200),
                prompt: egui::Color32::from_rgb(100, 50, 200),
            },
            "dracula" => TerminalTheme {
                background: egui::Color32::from_rgb(40, 42, 54),
                foreground: egui::Color32::from_rgb(248, 248, 242),
                selection: egui::Color32::from_rgb(68, 71, 90),
                cursor: egui::Color32::from_rgb(248, 248, 242),
                error: egui::Color32::from_rgb(255, 85, 85),
                success: egui::Color32::from_rgb(80, 250, 123),
                warning: egui::Color32::from_rgb(241, 250, 140),
                info: egui::Color32::from_rgb(139, 233, 253),
                prompt: egui::Color32::from_rgb(189, 147, 249),
            },
            _ => {
                self.console.write_error(&format!("Unknown theme: {}\n", theme_name));
                return;
            }
        };

        self.console.set_theme(theme);
        self.console.write_success(&format!("Applied {} theme\n", theme_name));
    }

    /// Get access to the underlying console for advanced operations
    pub fn console_mut(&mut self) -> &mut ConsoleWindow {
        &mut self.console
    }

    /// Get read-only access to the underlying console
    pub fn console(&self) -> &ConsoleWindow {
        &self.console
    }
} 