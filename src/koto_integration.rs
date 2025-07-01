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
                if args.len() >= 2 {
                    if let (Some(bg_str), Some(fg_str)) = (koto_value_as_str(&args[0]), koto_value_as_str(&args[1])) {
                        if let (Ok(bg), Ok(fg)) = (parse_color(&bg_str), parse_color(&fg_str)) {
                            let mut context = context.lock().unwrap();
                            context.egui_commands.push(EguiCommand::SetTheme {
                                background: bg,
                                foreground: fg,
                            });
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