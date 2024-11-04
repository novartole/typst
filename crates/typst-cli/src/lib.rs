mod args;
mod compile;
mod download;
mod fonts;
mod greet;
mod init;
mod package;
mod query;
mod terminal;
mod timings;
#[cfg(feature = "self-update")]
mod update;
mod watch;
mod world;

use std::cell::Cell;
use std::io::{self, Write};
use std::process::ExitCode;
use std::sync::{LazyLock, OnceLock};

use clap::error::ErrorKind;
use clap::Parser;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::WriteColor;
use typst::diag::HintedStrResult;

use crate::args::{CliArguments, Command};
use crate::timings::Timer;

thread_local! {
    /// The CLI's exit code.
    static EXIT: Cell<ExitCode> = const { Cell::new(ExitCode::SUCCESS) };
}

// The parsed command line arguments.
static ARGS: LazyLock<CliArguments> = LazyLock::new(|| ARGS2.get().unwrap().clone());
static ARGS2: OnceLock<CliArguments> = OnceLock::new();

/// Entry point.
pub fn exec(args: Vec<String>) -> ExitCode {
    let res = dispatch(args);

    if let Err(msg) = res {
        set_failed();
        print_error(msg.message()).expect("failed to print error");
    }

    EXIT.with(|cell| cell.get())
}

/// Execute the requested command.
fn dispatch(args: Vec<String>) -> HintedStrResult<()> {
    let args = ARGS2.get_or_init(|| {
        CliArguments::try_parse_from(args).unwrap_or_else(|error| {
            if error.kind() == ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand {
                crate::greet::greet();
            }
            error.exit();
        })
    });

    let timer = Timer::new(args);

    match &args.command {
        Command::Compile(command) => crate::compile::compile(timer, command.clone())?,
        // Command::Watch(command) => crate::watch::watch(timer, command.clone())?,
        Command::Init(command) => crate::init::init(command)?,
        Command::Query(command) => crate::query::query(command)?,
        // Command::Fonts(command) => crate::fonts::fonts(command),
        // Command::Update(command) => crate::update::update(command)?,
        _ => todo!("error handling of unsupported Command"),
    }

    Ok(())
}

/// Ensure a failure exit code.
fn set_failed() {
    EXIT.with(|cell| cell.set(ExitCode::FAILURE));
}

/// Used by `args.rs`.
fn typst_version() -> &'static str {
    env!("TYPST_VERSION")
}

/// Print an application-level error (independent from a source file).
fn print_error(msg: &str) -> io::Result<()> {
    let styles = term::Styles::default();

    let mut output = terminal::out();
    output.set_color(&styles.header_error)?;
    write!(output, "error")?;

    output.reset()?;
    writeln!(output, ": {msg}")
}
