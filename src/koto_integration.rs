use std::sync::{Arc, Mutex};
use koto::{Koto, KotoSettings};
use koto_runtime::{KValue, KMap, CallContext};
use egui::Color32;
use thiserror::Error;

/// Errors that can occur during koto execution
#[derive(Error, Debug)]
pub enum KotoError {
    #[error("Koto runtime error: {0}")]
    Runtime(String),
    #[error("Koto compilation error: {0}")]
    Compilation(String),
    #[error("Koto value conversion error: {0}")]
    Conversion(String),
}

/// Commands that can be sent from Koto to egui
#[derive(Debug, Clone)]
pub enum EguiCommand {
    SetTheme { 
        background: Color32, 
        foreground: Color32 
    },
    SetFullTheme { 
        theme: crate::TerminalTheme 
    },
    WriteLine { 
        text: String, 
        style: String 
    },
    ClearConsole,
    SetWindowTitle { 
        title: String 
    },
}

/// Context shared between Koto and the console
#[derive(Debug, Default)]
pub struct ConsoleContext {
    pub output_buffer: Vec<String>,
    pub error_buffer: Vec<String>,
    pub egui_commands: Vec<EguiCommand>,
}

/// Koto runtime manager for the console
pub struct KotoRuntime {
    koto: Koto,
    console_context: Arc<Mutex<ConsoleContext>>,
}

impl KotoRuntime {
    /// Create a new koto runtime with egui bindings
    pub fn new() -> Result<Self, KotoError> {
        let context = Arc::new(Mutex::new(ConsoleContext::default()));
        let context_clone = context.clone();
        
        let koto = Koto::with_settings(KotoSettings::default());

        // Add console print function
        {
            let context = context_clone.clone();
            koto.prelude().add_fn("print", move |ctx: &mut CallContext| -> koto_runtime::Result<KValue> {
                let mut context = context.lock().unwrap();
                let args = ctx.args();
                let output = args.iter()
                    .map(|arg| koto_value_to_string(arg))
                    .collect::<Vec<_>>()
                    .join(" ");
                context.output_buffer.push(output);
                Ok(KValue::Null)
            });
        }

        // Add console error function
        {
            let context = context_clone.clone();
            koto.prelude().add_fn("error", move |ctx: &mut CallContext| -> koto_runtime::Result<KValue> {
                let mut context = context.lock().unwrap();
                let args = ctx.args();
                let output = args.iter()
                    .map(|arg| koto_value_to_string(arg))
                    .collect::<Vec<_>>()
                    .join(" ");
                context.error_buffer.push(output);
                Ok(KValue::Null)
            });
        }

        // Add egui theme function
        {
            let context = context_clone.clone();
            koto.prelude().add_fn("set_theme", move |ctx: &mut CallContext| -> koto_runtime::Result<KValue> {
                let args = ctx.args();
                let mut context = context.lock().unwrap();
                
                if args.is_empty() {
                    return Ok(KValue::Null);
                }

                // Case 1: Single argument - preset theme name
                if args.len() == 1 {
                    if let Some(theme_name) = koto_value_as_str(&args[0]) {
                        let theme = match theme_name.to_lowercase().as_str() {
                            "dark" => Some(create_dark_theme()),
                            "light" => Some(create_light_theme()),
                            "dracula" => Some(create_dracula_theme()),
                            "solarized" => Some(create_solarized_theme()),
                            "nord" => Some(create_nord_theme()),
                            "matrix" => Some(create_matrix_theme()),
                            "ocean" => Some(create_ocean_theme()),
                            "cyberpunk" => Some(create_cyberpunk_theme()),
                            _ => None,
                        };

                        if let Some(theme) = theme {
                            context.egui_commands.push(EguiCommand::SetFullTheme { theme });
                        }
                    }
                    return Ok(KValue::Null);
                }

                // Case 2: Two arguments - background and foreground colors (backward compatibility)
                if args.len() == 2 {
                    if let (Some(bg_str), Some(fg_str)) = (koto_value_as_str(&args[0]), koto_value_as_str(&args[1])) {
                        if let (Ok(bg), Ok(fg)) = (parse_color(&bg_str), parse_color(&fg_str)) {
                            context.egui_commands.push(EguiCommand::SetTheme {
                                background: bg,
                                foreground: fg,
                            });
                        }
                    }
                    return Ok(KValue::Null);
                }

                // Case 3: Map argument - full theme configuration
                if args.len() == 1 {
                    if let KValue::Map(theme_map) = &args[0] {
                        let theme = parse_theme_from_map(theme_map);
                        if let Some(theme) = theme {
                            context.egui_commands.push(EguiCommand::SetFullTheme { theme });
                        }
                    }
                }

                Ok(KValue::Null)
            });
        }

        // Add styled output function
        {
            let context = context_clone.clone();
            koto.prelude().add_fn("write_styled", move |ctx: &mut CallContext| -> koto_runtime::Result<KValue> {
                let args = ctx.args();
                if args.len() >= 2 {
                    if let (Some(text), Some(style)) = (koto_value_as_str(&args[0]), koto_value_as_str(&args[1])) {
                        let mut context = context.lock().unwrap();
                        context.egui_commands.push(EguiCommand::WriteLine {
                            text,
                            style,
                        });
                    }
                }
                Ok(KValue::Null)
            });
        }

        // Add clear console function
        {
            let context = context_clone.clone();
            koto.prelude().add_fn("clear_console", move |_ctx: &mut CallContext| -> koto_runtime::Result<KValue> {
                let mut context = context.lock().unwrap();
                context.egui_commands.push(EguiCommand::ClearConsole);
                Ok(KValue::Null)
            });
        }

        // Add console utilities module
        let console_module = create_console_module(context_clone.clone());
        koto.prelude().insert("console", console_module);

        Ok(Self {
            koto,
            console_context: context,
        })
    }

    /// Execute koto code and return the result
    pub fn execute(&mut self, code: &str) -> Result<String, KotoError> {
        // Clear previous output buffers
        {
            let mut context = self.console_context.lock().unwrap();
            context.output_buffer.clear();
            context.error_buffer.clear();
            context.egui_commands.clear();
        }

        // Execute the code
        let result = self.koto.compile_and_run(code)
            .map_err(|e| KotoError::Runtime(e.to_string()))?;

        // Collect outputs
        let context = self.console_context.lock().unwrap();
        let mut output = String::new();
        
        // Add print outputs
        for line in &context.output_buffer {
            output.push_str(line);
            output.push('\n');
        }

        // Add error outputs
        for line in &context.error_buffer {
            output.push_str(&format!("Error: {}\n", line));
        }

        // Add result if it's not null
        if !matches!(result, KValue::Null) {
            output.push_str(&format!("=> {}\n", koto_value_to_string(&result)));
        }

        Ok(output)
    }

    /// Get and clear egui commands from the context
    pub fn get_egui_commands(&self) -> Vec<EguiCommand> {
        let mut context = self.console_context.lock().unwrap();
        std::mem::take(&mut context.egui_commands)
    }

    /// Add a global variable to the koto runtime
    pub fn set_global(&mut self, name: &str, value: String) {
        let koto_value = KValue::Str(value.into());
        self.koto.prelude().insert(name, koto_value);
    }

    /// Load and execute a koto file
    pub fn load_file(&mut self, path: &str) -> Result<String, KotoError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| KotoError::Runtime(format!("Failed to read file: {}", e)))?;
        self.execute(&content)
    }
}

/// Create a console utilities module for koto
fn create_console_module(context: Arc<Mutex<ConsoleContext>>) -> KValue {
    let module = KMap::new();

    // Add info function
    {
        let context = context.clone();
        module.add_fn("info", move |ctx: &mut CallContext| -> koto_runtime::Result<KValue> {
            let mut context = context.lock().unwrap();
            let args = ctx.args();
            let text = args.iter()
                .map(|arg| koto_value_to_string(arg))
                .collect::<Vec<_>>()
                .join(" ");
            context.egui_commands.push(EguiCommand::WriteLine {
                text,
                style: "info".to_string(),
            });
            Ok(KValue::Null)
        });
    }

    // Add success function
    {
        let context = context.clone();
        module.add_fn("success", move |ctx: &mut CallContext| -> koto_runtime::Result<KValue> {
            let mut context = context.lock().unwrap();
            let args = ctx.args();
            let text = args.iter()
                .map(|arg| koto_value_to_string(arg))
                .collect::<Vec<_>>()
                .join(" ");
            context.egui_commands.push(EguiCommand::WriteLine {
                text,
                style: "success".to_string(),
            });
            Ok(KValue::Null)
        });
    }

    // Add warning function
    {
        let context = context.clone();
        module.add_fn("warning", move |ctx: &mut CallContext| -> koto_runtime::Result<KValue> {
            let mut context = context.lock().unwrap();
            let args = ctx.args();
            let text = args.iter()
                .map(|arg| koto_value_to_string(arg))
                .collect::<Vec<_>>()
                .join(" ");
            context.egui_commands.push(EguiCommand::WriteLine {
                text,
                style: "warning".to_string(),
            });
            Ok(KValue::Null)
        });
    }

    // Add error function
    {
        let context = context.clone();
        module.add_fn("error", move |ctx: &mut CallContext| -> koto_runtime::Result<KValue> {
            let mut context = context.lock().unwrap();
            let args = ctx.args();
            let text = args.iter()
                .map(|arg| koto_value_to_string(arg))
                .collect::<Vec<_>>()
                .join(" ");
            context.egui_commands.push(EguiCommand::WriteLine {
                text,
                style: "error".to_string(),
            });
            Ok(KValue::Null)
        });
    }

    KValue::Map(module)
}

/// Convert KValue to string representation
fn koto_value_to_string(value: &KValue) -> String {
    match value {
        KValue::Null => "null".to_string(),
        KValue::Bool(b) => b.to_string(),
        KValue::Number(n) => n.to_string(),
        KValue::Str(s) => s.to_string(),
        KValue::List(list) => {
            let items: Vec<String> = list.data().iter()
                .map(|item| koto_value_to_string(item))
                .collect();
            format!("[{}]", items.join(", "))
        }
        KValue::Map(map) => {
            let items: Vec<String> = map.data().iter()
                .map(|(k, v)| format!("{}: {}", k, koto_value_to_string(v)))
                .collect();
            format!("{{{}}}", items.join(", "))
        }
        _ => format!("{:?}", value),
    }
}

/// Extract string from KValue if possible
fn koto_value_as_str(value: &KValue) -> Option<String> {
    match value {
        KValue::Str(s) => Some(s.to_string()),
        _ => None,
    }
}

/// Parse a color string (hex format like "#RGB" or "#RRGGBB")
fn parse_color(color_str: &str) -> Result<Color32, KotoError> {
    if !color_str.starts_with('#') {
        return Err(KotoError::Conversion("Color must start with #".to_string()));
    }

    let hex = &color_str[1..];
    match hex.len() {
        3 => {
            // #RGB format
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16)
                .map_err(|_| KotoError::Conversion("Invalid hex color".to_string()))?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16)
                .map_err(|_| KotoError::Conversion("Invalid hex color".to_string()))?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16)
                .map_err(|_| KotoError::Conversion("Invalid hex color".to_string()))?;
            Ok(Color32::from_rgb(r, g, b))
        }
        6 => {
            // #RRGGBB format
            let r = u8::from_str_radix(&hex[0..2], 16)
                .map_err(|_| KotoError::Conversion("Invalid hex color".to_string()))?;
            let g = u8::from_str_radix(&hex[2..4], 16)
                .map_err(|_| KotoError::Conversion("Invalid hex color".to_string()))?;
            let b = u8::from_str_radix(&hex[4..6], 16)
                .map_err(|_| KotoError::Conversion("Invalid hex color".to_string()))?;
            Ok(Color32::from_rgb(r, g, b))
        }
        _ => Err(KotoError::Conversion("Color must be #RGB or #RRGGBB format".to_string())),
    }
}

/// Create preset themes
fn create_dark_theme() -> crate::TerminalTheme {
    crate::TerminalTheme {
        background: Color32::from_rgb(30, 30, 30),
        foreground: Color32::from_rgb(220, 220, 220),
        selection: Color32::from_rgb(70, 70, 70),
        cursor: Color32::from_rgb(255, 255, 255),
        error: Color32::from_rgb(255, 85, 85),
        success: Color32::from_rgb(80, 250, 123),
        warning: Color32::from_rgb(255, 184, 108),
        info: Color32::from_rgb(139, 233, 253),
        prompt: Color32::from_rgb(189, 147, 249),
    }
}

fn create_light_theme() -> crate::TerminalTheme {
    crate::TerminalTheme {
        background: Color32::from_rgb(240, 240, 240),
        foreground: Color32::from_rgb(30, 30, 30),
        selection: Color32::from_rgb(180, 180, 180),
        cursor: Color32::from_rgb(0, 0, 0),
        error: Color32::from_rgb(220, 50, 50),
        success: Color32::from_rgb(0, 170, 0),
        warning: Color32::from_rgb(200, 150, 0),
        info: Color32::from_rgb(0, 100, 200),
        prompt: Color32::from_rgb(100, 50, 200),
    }
}

fn create_dracula_theme() -> crate::TerminalTheme {
    crate::TerminalTheme {
        background: Color32::from_rgb(40, 42, 54),
        foreground: Color32::from_rgb(248, 248, 242),
        selection: Color32::from_rgb(68, 71, 90),
        cursor: Color32::from_rgb(248, 248, 242),
        error: Color32::from_rgb(255, 85, 85),
        success: Color32::from_rgb(80, 250, 123),
        warning: Color32::from_rgb(241, 250, 140),
        info: Color32::from_rgb(139, 233, 253),
        prompt: Color32::from_rgb(189, 147, 249),
    }
}

fn create_solarized_theme() -> crate::TerminalTheme {
    crate::TerminalTheme {
        background: Color32::from_rgb(0, 43, 54),
        foreground: Color32::from_rgb(131, 148, 150),
        selection: Color32::from_rgb(7, 54, 66),
        cursor: Color32::from_rgb(131, 148, 150),
        error: Color32::from_rgb(220, 50, 47),
        success: Color32::from_rgb(133, 153, 0),
        warning: Color32::from_rgb(181, 137, 0),
        info: Color32::from_rgb(38, 139, 210),
        prompt: Color32::from_rgb(108, 113, 196),
    }
}

fn create_nord_theme() -> crate::TerminalTheme {
    crate::TerminalTheme {
        background: Color32::from_rgb(46, 52, 64),
        foreground: Color32::from_rgb(216, 222, 233),
        selection: Color32::from_rgb(67, 76, 94),
        cursor: Color32::from_rgb(216, 222, 233),
        error: Color32::from_rgb(191, 97, 106),
        success: Color32::from_rgb(163, 190, 140),
        warning: Color32::from_rgb(235, 203, 139),
        info: Color32::from_rgb(129, 161, 193),
        prompt: Color32::from_rgb(180, 142, 173),
    }
}

fn create_matrix_theme() -> crate::TerminalTheme {
    crate::TerminalTheme {
        background: Color32::from_rgb(0, 0, 0),
        foreground: Color32::from_rgb(0, 255, 65),
        selection: Color32::from_rgb(0, 100, 25),
        cursor: Color32::from_rgb(0, 255, 65),
        error: Color32::from_rgb(255, 65, 65),
        success: Color32::from_rgb(0, 255, 100),
        warning: Color32::from_rgb(255, 255, 0),
        info: Color32::from_rgb(100, 255, 150),
        prompt: Color32::from_rgb(0, 200, 50),
    }
}

fn create_ocean_theme() -> crate::TerminalTheme {
    crate::TerminalTheme {
        background: Color32::from_rgb(15, 42, 66),
        foreground: Color32::from_rgb(171, 196, 219),
        selection: Color32::from_rgb(45, 72, 96),
        cursor: Color32::from_rgb(171, 196, 219),
        error: Color32::from_rgb(255, 102, 102),
        success: Color32::from_rgb(102, 204, 204),
        warning: Color32::from_rgb(255, 204, 102),
        info: Color32::from_rgb(102, 153, 255),
        prompt: Color32::from_rgb(153, 102, 255),
    }
}

fn create_cyberpunk_theme() -> crate::TerminalTheme {
    crate::TerminalTheme {
        background: Color32::from_rgb(16, 0, 43),
        foreground: Color32::from_rgb(255, 20, 147),
        selection: Color32::from_rgb(75, 0, 130),
        cursor: Color32::from_rgb(255, 20, 147),
        error: Color32::from_rgb(255, 69, 0),
        success: Color32::from_rgb(50, 205, 50),
        warning: Color32::from_rgb(255, 215, 0),
        info: Color32::from_rgb(0, 191, 255),
        prompt: Color32::from_rgb(238, 130, 238),
    }
}

/// Parse theme from Koto map
fn parse_theme_from_map(theme_map: &KMap) -> Option<crate::TerminalTheme> {
    let background = theme_map.get("background")
        .and_then(|v| koto_value_as_str(&v))
        .and_then(|s| parse_color(&s).ok())?;
    
    let foreground = theme_map.get("foreground")
        .and_then(|v| koto_value_as_str(&v))
        .and_then(|s| parse_color(&s).ok())?;

    let selection = theme_map.get("selection")
        .and_then(|v| koto_value_as_str(&v))
        .and_then(|s| parse_color(&s).ok())
        .unwrap_or(Color32::from_rgb(70, 70, 70));

    let cursor = theme_map.get("cursor")
        .and_then(|v| koto_value_as_str(&v))
        .and_then(|s| parse_color(&s).ok())
        .unwrap_or(foreground);

    let error = theme_map.get("error")
        .and_then(|v| koto_value_as_str(&v))
        .and_then(|s| parse_color(&s).ok())
        .unwrap_or(Color32::from_rgb(255, 85, 85));

    let success = theme_map.get("success")
        .and_then(|v| koto_value_as_str(&v))
        .and_then(|s| parse_color(&s).ok())
        .unwrap_or(Color32::from_rgb(80, 250, 123));

    let warning = theme_map.get("warning")
        .and_then(|v| koto_value_as_str(&v))
        .and_then(|s| parse_color(&s).ok())
        .unwrap_or(Color32::from_rgb(255, 184, 108));

    let info = theme_map.get("info")
        .and_then(|v| koto_value_as_str(&v))
        .and_then(|s| parse_color(&s).ok())
        .unwrap_or(Color32::from_rgb(139, 233, 253));

    let prompt = theme_map.get("prompt")
        .and_then(|v| koto_value_as_str(&v))
        .and_then(|s| parse_color(&s).ok())
        .unwrap_or(Color32::from_rgb(189, 147, 249));

    Some(crate::TerminalTheme {
        background,
        foreground,
        selection,
        cursor,
        error,
        success,
        warning,
        info,
        prompt,
    })
} 