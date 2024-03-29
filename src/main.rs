#![cfg_attr(feature = "deny-warnings", deny(warnings))]
// warn on lints, that are included in `rust-lang/rust`s bootstrap
#![warn(rust_2018_idioms, unused_lifetimes)]

use rustc_tools_util::VersionInfo;
use std::env;
use std::path::PathBuf;
use std::process::{self, Command};

mod docs;

const CARGO_SUBSTRACE_HELP: &str = r#"Substrace: Checks a package to catch common mistakes and improve your Rust code.

Usage:
    cargo substrace [options] [--] [<opts>...]

Common options:
    --no-deps                Run Substrace only on the given crate, without linting the dependencies
    --fix                    Automatically apply lint suggestions. This flag implies `--no-deps`
    -h, --help               Print this message
    -V, --version            Print version info and exit
    --explain LINT           Print the documentation for a given lint

Other options are the same as `cargo check`.

To allow or deny a lint from the command line you can use `cargo substrace --`
with:

    -W --warn OPT       Set lint warnings
    -A --allow OPT      Set lint allowed
    -D --deny OPT       Set lint denied
    -F --forbid OPT     Set lint forbidden

You can use tool lints to allow or deny lints from your code, eg.:

    #[allow(substrace::panics)]
"#;

fn show_help() {
    println!("{}", CARGO_SUBSTRACE_HELP);
}

fn show_version() {
    let version_info = rustc_tools_util::get_version_info!();
    println!("{}", version_info);
}

pub fn main() {
    println!("Starting Substrace...");

    // Check for version and help flags even when invoked as 'cargo-substrace'
    if env::args().any(|a| a == "--help" || a == "-h") {
        show_help();
        return;
    }

    if env::args().any(|a| a == "--version" || a == "-V") {
        show_version();
        return;
    }

    if let Some(pos) = env::args().position(|a| a == "--explain") {
        if let Some(mut lint) = env::args().nth(pos + 1) {
            println!("Lint? {:?}", lint);
            lint.make_ascii_lowercase();
            docs::explain(&lint.strip_prefix("substrace::").unwrap_or(&lint).replace('-', "_"));
        } else {
            show_help();
        }
        return;
    }

    if let Err(code) = process(env::args().skip(2)) {
        process::exit(code);
    }
}

struct SubstraceCmd {
    cargo_subcommand: &'static str,
    args: Vec<String>,
    substrace_args: Vec<String>,
}

impl SubstraceCmd {
    fn new<I>(mut old_args: I) -> Self
    where
        I: Iterator<Item = String>,
    {
        let mut cargo_subcommand = "check";
        let mut args = vec![];
        let mut substrace_args: Vec<String> = vec![];

        for arg in old_args.by_ref() {
            match arg.as_str() {
                "--fix" => {
                    cargo_subcommand = "fix";
                    continue;
                },
                "--no-deps" => {
                    substrace_args.push("--no-deps".into());
                    continue;
                },
                "--" => break,
                _ => {},
            }

            args.push(arg);
        }

        substrace_args.append(&mut (old_args.collect()));
        if cargo_subcommand == "fix" && !substrace_args.iter().any(|arg| arg == "--no-deps") {
            substrace_args.push("--no-deps".into());
        }

        Self {
            cargo_subcommand,
            args,
            substrace_args,
        }
    }

    fn path() -> PathBuf {
        let mut path = env::current_exe()
            .expect("current executable path invalid")
            .with_file_name("substrace-driver");

        if cfg!(windows) {
            path.set_extension("exe");
        }

        path
    }

    fn into_std_cmd(self) -> Command {
        let mut cmd = Command::new("cargo");
        let substrace_args: String = self
            .substrace_args
            .iter()
            .map(|arg| format!("{}__SUBSTRACE_HACKERY__", arg))
            .collect();

        // Currently, `SUBSTRACE_TERMINAL_WIDTH` is used only to format "unknown field" error messages.
        let terminal_width = termize::dimensions().map_or(0, |(w, _)| w);

        cmd.env("RUSTC_WORKSPACE_WRAPPER", Self::path())
            .env("SUBSTRACE_ARGS", substrace_args)
            .env("SUBSTRACE_TERMINAL_WIDTH", terminal_width.to_string())
            .arg(self.cargo_subcommand)
            .args(&self.args);

        cmd
    }
}

fn process<I>(old_args: I) -> Result<(), i32>
where
    I: Iterator<Item = String>,
{
    let cmd = SubstraceCmd::new(old_args);

    let mut cmd = cmd.into_std_cmd();

    println!("cmd: {:?}", cmd);

    let exit_status = cmd
        .spawn()
        .expect("could not run cargo")
        .wait()
        .expect("failed to wait for cargo?");

    if exit_status.success() {
        Ok(())
    } else {
        Err(exit_status.code().unwrap_or(-1))
    }
}

#[cfg(test)]
mod tests {
    use super::SubstraceCmd;

    #[test]
    fn fix() {
        let args = "cargo substrace --fix".split_whitespace().map(ToString::to_string);
        let cmd = SubstraceCmd::new(args);
        assert_eq!("fix", cmd.cargo_subcommand);
        assert!(!cmd.args.iter().any(|arg| arg.ends_with("unstable-options")));
    }

    #[test]
    fn fix_implies_no_deps() {
        let args = "cargo substrace --fix".split_whitespace().map(ToString::to_string);
        let cmd = SubstraceCmd::new(args);
        assert!(cmd.substrace_args.iter().any(|arg| arg == "--no-deps"));
    }

    #[test]
    fn no_deps_not_duplicated_with_fix() {
        let args = "cargo substrace --fix -- --no-deps"
            .split_whitespace()
            .map(ToString::to_string);
        let cmd = SubstraceCmd::new(args);
        assert_eq!(cmd.substrace_args.iter().filter(|arg| *arg == "--no-deps").count(), 1);
    }

    #[test]
    fn check() {
        let args = "cargo substrace".split_whitespace().map(ToString::to_string);
        let cmd = SubstraceCmd::new(args);
        assert_eq!("check", cmd.cargo_subcommand);
    }
}
