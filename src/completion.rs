use std::collections::HashMap;
use crate::ConsoleWindow;

/// Support for intelligent command completion beyond simple prefix matching
#[derive(Debug)]
pub struct IntelligentCompletion {
    /// Maps command aliases to their canonical names
    aliases: HashMap<String, String>,
    /// Maps commands to their subcommands and arguments
    command_structure: HashMap<String, CommandInfo>,
}

#[derive(Debug)]
pub struct CommandInfo {
    pub description: String,
    pub subcommands: Vec<String>,
    pub arguments: Vec<ArgumentInfo>,
}

#[derive(Debug)]
pub struct ArgumentInfo {
    pub name: String,
    pub description: String,
    pub is_required: bool,
}

impl Default for IntelligentCompletion {
    fn default() -> Self {
        Self {
            aliases: HashMap::new(),
            command_structure: HashMap::new(),
        }
    }
}

impl IntelligentCompletion {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a command with its aliases
    pub fn register_command(&mut self, command: &str, aliases: &[&str], description: &str) {
        // Add the command to the structure map
        self.command_structure.insert(
            command.to_string(), 
            CommandInfo {
                description: description.to_string(),
                subcommands: Vec::new(),
                arguments: Vec::new(),
            }
        );

        // Register all aliases
        for alias in aliases {
            self.aliases.insert(alias.to_string(), command.to_string());
        }
    }

    /// Register an argument for a command
    pub fn register_argument(
        &mut self,
        command: &str, 
        arg_name: &str, 
        description: &str, 
        required: bool
    ) {
        if let Some(cmd_info) = self.command_structure.get_mut(command) {
            cmd_info.arguments.push(ArgumentInfo {
                name: arg_name.to_string(),
                description: description.to_string(),
                is_required: required,
            });
        }
    }

    /// Register a subcommand for a parent command
    pub fn register_subcommand(&mut self, parent: &str, subcommand: &str) {
        if let Some(cmd_info) = self.command_structure.get_mut(parent) {
            cmd_info.subcommands.push(subcommand.to_string());
        }
    }

    /// Get completion suggestions based on current command line
    pub fn get_suggestions(&self, input: &str) -> Vec<String> {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();

        if parts.is_empty() {
            // Empty input, return all commands
            return self.command_structure.keys().cloned().collect();
        }

        let first_word = parts[0];

        // If the input is just the first word (possibly incomplete)
        if parts.len() == 1 {
            // Filter commands that start with the input
            let mut matches: Vec<String> = self.command_structure.keys()
                .filter(|cmd| cmd.starts_with(first_word))
                .cloned()
                .collect();

            // Add aliases that match
            for (alias, _cmd) in &self.aliases {
                if alias.starts_with(first_word) {
                    matches.push(alias.clone());
                }
            }

            // Deduplicate and sort
            matches.sort();
            matches.dedup();
            return matches;
        }

        // If we're completing an argument for a known command
        let command = if let Some(cmd) = self.resolve_command(first_word) {
            cmd
        } else {
            return Vec::new();
        };

        if let Some(cmd_info) = self.command_structure.get(&command) {
            // If command has subcommands and we're at position to complete them
            if parts.len() == 2 && !cmd_info.subcommands.is_empty() {
                return cmd_info.subcommands.iter()
                    .filter(|sub| sub.starts_with(parts[1]))
                    .cloned()
                    .collect();
            }

            // If command has arguments, suggest them
            if !cmd_info.arguments.is_empty() {
                return cmd_info.arguments.iter()
                    .map(|arg| format!("{}{}", 
                        if arg.is_required { "" } else { "[" },
                        arg.name
                    ))
                    .collect();
            }
        }

        Vec::new()
    }

    /// Resolve a command or alias to its canonical name
    fn resolve_command(&self, input: &str) -> Option<String> {
        // Check if it's a direct command
        if self.command_structure.contains_key(input) {
            return Some(input.to_string());
        }

        // Check if it's an alias
        if let Some(cmd) = self.aliases.get(input) {
            return Some(cmd.clone());
        }

        None
    }

    /// Get help text for a command
    pub fn get_help_text(&self, command: &str) -> Option<String> {
        let cmd = self.resolve_command(command)?;
        let cmd_info = self.command_structure.get(&cmd)?;

        let mut help = format!("{} - {}\n\n", cmd, cmd_info.description);

        if !cmd_info.arguments.is_empty() {
            help.push_str("Arguments:\n");
            for arg in &cmd_info.arguments {
                help.push_str(&format!("  {}{} - {}\n", 
                    if arg.is_required { "" } else { "[" },
                    arg.name,
                    arg.description
                ));
            }
            help.push('\n');
        }

        if !cmd_info.subcommands.is_empty() {
            help.push_str("Subcommands:\n");
            for sub in &cmd_info.subcommands {
                if let Some(sub_info) = self.command_structure.get(sub) {
                    help.push_str(&format!("  {} - {}\n", sub, sub_info.description));
                } else {
                    help.push_str(&format!("  {}\n", sub));
                }
            }
        }

        Some(help)
    }
}

impl ConsoleWindow {
    /// Get intelligent command completion suggestions
    pub fn get_intelligent_suggestions(&self, input: &str) -> Vec<String> {
        // If we have an intelligent completion engine, use it
        if let Some(completion) = &self.intelligent_completion {
            return completion.get_suggestions(input);
        }

        // Fallback to simple prefix matching
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return self.tab_command_table.clone();
        }

        self.tab_command_table.iter()
            .filter(|cmd| cmd.starts_with(trimmed))
            .cloned()
            .collect()
    }
}
