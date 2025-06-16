# Enhanced Terminal for egui

Provides a feature-rich terminal window for egui applications. This is not a shell to the OS; it's simply a command shell window with modern terminal features. It's very useful for providing a command line interface inside a GUI app.

## Features

- Modern terminal experience with syntax highlighting
- Multiple built-in themes (dark, light, dracula, solarized, nord)
- Colored output with support for success, error, warning, and info styling
- Persistent, searchable command history
- Tab completion for filesystem paths and commands
- Host in any container with flexible layout options

## Demo

Run it with `cargo run -p demo`. Type 'help' at the command prompt to see available commands.

## Usage

You need a ConsoleWindow instance in your egui App:

```rust
pub struct MyApp {
    // Your app fields
    console: ConsoleWindow,
}
```

And then use the builder to instantiate a ConsoleWindow with your preferred settings:

```rust
ConsoleBuilder::new()
    .prompt(">> ")
    .history_size(20)
    .tab_quote_character('"')
    .theme(TerminalTheme::default()) // or use a custom theme
    .build();
```

On each UI update cycle, call the draw method, passing in the UI instance that should host the console window. Draw returns a ConsoleEvent enum that you can use to respond to user input:

```rust
let console_response = self.console.draw(ui);
if let ConsoleEvent::Command(command) = console_response {
    // Process the command
    self.console.write_success("Command processed successfully!");
    self.console.prompt();
}
```

### Styled Output

The enhanced terminal supports different text styles:

```rust
// Regular output
console.write("Regular text\n");

// Styled output
console.write_error("Error message\n");
console.write_success("Success message\n");
console.write_warning("Warning message\n");
console.write_info("Information message\n");
```

### Themes

The terminal supports multiple built-in themes and custom theming:

```rust
// Create a custom theme
let custom_theme = TerminalTheme {
    background: egui::Color32::from_rgb(30, 30, 30),
    foreground: egui::Color32::from_rgb(220, 220, 220),
    // ... other theme properties
};

// Apply when creating the console
ConsoleBuilder::new()
    .theme(custom_theme)
    .build();
```

### Command Completion

Tab completion works for commands and filesystem paths. The console window maintains a `Vec<String>` of commands. You can modify this table by calling the `command_table_mut()` method.

## License

This project is licensed under the same terms as egui itself.

![image](https://github.com/user-attachments/assets/de2df396-68ac-4723-ae62-2811fb81ba05)
