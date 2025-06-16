use clap::Command;
use clap::{arg, Arg};

// Clap sub command syntax defintions
pub fn syntax() -> Command {
    // strip out usage
    const PARSER_TEMPLATE: &str = "\
        {all-args}
    ";
    // strip out name/version
    const APPLET_TEMPLATE: &str = "\
        {about-with-newline}\n\
        {usage-heading}\n    {usage}\n\
        \n\
        {all-args}{after-help}\
    ";

    Command::new("enhanced-terminal")
        .multicall(true)
        .arg_required_else_help(true)
        .subcommand_required(true)
        .subcommand_value_name("Command")
        .subcommand_help_heading("Commands")
        .help_template(PARSER_TEMPLATE)
        .subcommand(
            Command::new("quit")
                .visible_aliases(["exit", "q"])
                .about("Quit terminal")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("dir")
                .visible_aliases(["ls", "list"])
                .about("Directory list of current directory")
                .arg(arg!([filter]))
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("dark")
                .about("Set dark mode")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("light")
                .about("Set light mode")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("clear_screen")
                .visible_aliases(["cls", "clear"])
                .about("Clear the screen")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("history")
                .about("Display command history")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("clear_history")
                .help_template(APPLET_TEMPLATE)
                .visible_aliases(["clh"]),
        )
        .subcommand(
            Command::new("cd")
                .about("Change current directory")
                .arg(Arg::new("directory").required(true))
                .arg_required_else_help(true)
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("cat")
                .about("Display file contents")
                .arg(Arg::new("file").required(true))
                .arg_required_else_help(true)
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("pwd")
                .about("Print working directory")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("echo")
                .about("Echo text to console")
                .arg(arg!([text] ... "Text to echo"))
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("commands")
                .about("Display available commands")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("theme")
                .about("Change terminal theme")
                .arg(arg!([name] "Theme name (dark, light, dracula, solarized, nord)"))
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("codeview")
                .about("View code with syntax highlighting")
                .arg(Arg::new("language").required(true))
                .arg(Arg::new("file").required(true))
                .help_template(APPLET_TEMPLATE),
        )
}
