//! md2pdf command-line interface.

mod args;
mod commands;

use anyhow::Result;
use clap::Parser;

use args::{Cli, Command, ThemeCommand};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Convert(args)) => commands::convert::run(args),
        Some(Command::Validate { inputs, strict }) => commands::validate::run(&inputs, strict),
        Some(Command::Doctor) => commands::doctor::run(),
        Some(Command::Init { dir }) => commands::init::run(&dir),
        Some(Command::Theme(ThemeCommand::List)) => commands::theme::list(),
        Some(Command::Theme(ThemeCommand::Create { name })) => commands::theme::create(&name),
        Some(Command::Completions { shell }) => {
            use clap::CommandFactory;
            clap_complete::generate(shell, &mut Cli::command(), "md2pdf", &mut std::io::stdout());
            Ok(())
        }
        Some(Command::Man) => {
            use clap::CommandFactory;
            clap_mangen::Man::new(Cli::command()).render(&mut std::io::stdout())?;
            Ok(())
        }
        None => {
            if cli.convert.input.is_some() {
                commands::convert::run(cli.convert)
            } else {
                // No subcommand and no input: show help.
                use clap::CommandFactory;
                Cli::command().print_help()?;
                println!();
                Ok(())
            }
        }
    }
}
