#![deny(unused_must_use)]
#![deny(mutable_borrow_reservation_conflict)]
#![allow(clippy::cast_lossless)]

use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;

use structopt::{self, StructOpt};

mod constants;
mod expr;
mod format_value;
mod functions;
mod options;
mod template;
mod value;

use self::constants::{Constant, Constants};
use self::expr::EvalError;
use self::options::Options;
use self::value::Context;

#[derive(Debug, StructOpt, Default)]
#[structopt(author, about)]
#[structopt(rename_all = "kebab-case")]
pub struct Config {
    /// Target directory for generated files
    #[structopt(short, long, parse(from_os_str))]
    pub target_dir: PathBuf,

    /// Target filename stem
    #[structopt(short, long, parse(from_os_str), default_value = "constants")]
    pub stem: OsString,

    /// File specifying generation options
    #[structopt(short, long, parse(from_os_str))]
    pub options_file: PathBuf,

    /// File specifying constants
    #[structopt(short, long, parse(from_os_str))]
    pub constants_file: Vec<PathBuf>,
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Evaluation(String, EvalError),
    DuplicateConstant(String),
    Formatter(String),
    ImportsNotSupported { language: String },
    TypeRequired { language: String, constant: String },
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::Evaluation(name, error) => write!(f, "In constant {:?}: {}", name, error),
            Self::DuplicateConstant(name) => write!(f, "Duplicate constant definition {:?}", name),
            Self::ImportsNotSupported { language } => write!(
                f,
                "Language {:?} does not specify import syntax, but it is required",
                language
            ),
            Self::TypeRequired { language, constant } => write!(
                f,
                "Language {:?} requires types, but constant {:?} does not provide one",
                language, constant
            ),
            _ => write!(f, "{:?}", self),
        }
    }
}
impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
impl From<(Constant, EvalError)> for Error {
    fn from((c, error): (Constant, EvalError)) -> Self {
        Self::Evaluation(c.name, error)
    }
}

#[paw::main]
fn main(args: Config) {
    pretty_env_logger::init();
    if let Err(e) = inner_main(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn inner_main(args: Config) -> Result<(), Error> {
    let c = fs::read(args.options_file).unwrap();
    let opts: Options = toml::from_slice(&c).unwrap();

    let mut constants = Vec::new();
    for p in args.constants_file {
        let c = fs::read(p).unwrap();
        let t: Constants = toml::from_slice(&c).unwrap();
        constants.extend(t.constants);
    }

    // Resolve constant values
    let mut context: Context = Context::new();
    for constant in constants.iter_mut() {
        if context.contains_key(&constant.name) {
            return Err(Error::DuplicateConstant(constant.name.clone()));
        }
        constant
            .resolve_value(&context)
            .map_err(|err| (constant.clone(), err))?;
        context.insert(constant.name.clone(), constant.value());
    }

    // Generate files to memory
    let outputs = opts
        .languages()
        .into_iter()
        .map(|(lang_name, lang_opts)| {
            log::info!("Processing target {}", lang_name);
            let mut buffer = String::new();

            // Imports
            if opts.codegen.comment_sections {
                buffer.push_str(&lang_opts.format_comment("Imports"));
            }
            let mut imports: Vec<String> = constants
                .iter()
                .flat_map(|c| lang_opts.constant_imports(c))
                .collect();
            imports.sort();
            imports.dedup();
            for import in &imports {
                buffer.push_str(&lang_opts.format_import(import).ok_or_else(|| {
                    Error::ImportsNotSupported {
                        language: lang_name.to_owned(),
                    }
                })?);
                buffer.push('\n');
            }

            // Intro
            if opts.codegen.comment_sections {
                buffer.push_str(&lang_opts.format_comment("Start body block"));
            }
            buffer.push_str(&lang_opts.format_intro());

            // Actual constant values
            if opts.codegen.comment_sections {
                buffer.push_str(&lang_opts.format_comment("Constants"));
            }
            for constant in &constants {
                buffer.push_str(&lang_opts.format_constant(constant).ok_or_else(|| {
                    Error::TypeRequired {
                        language: lang_name.to_owned(),
                        constant: constant.name.to_owned(),
                    }
                })?);
                buffer.push('\n');
            }

            // Outro
            if opts.codegen.comment_sections {
                buffer.push_str(&lang_opts.format_comment("End body block"));
            }
            buffer.push_str(&lang_opts.format_outro());

            // Run formatter if available
            if let Some(f) = &lang_opts.formatter {
                buffer = run_formatter(f, &buffer)?;
            }

            Ok((lang_name, lang_opts, buffer))
        })
        .collect::<Result<Vec<_>, Error>>()?;

    // Actually write generated files
    for (lang_name, lang_opts, buffer) in outputs.into_iter() {
        let target_file = args.target_dir.join(format!(
            "{}{}",
            args.stem.to_str().unwrap(),
            lang_opts.file_ext
        ));
        log::info!("Writing {} file: {:?}", lang_name, target_file);
        fs::write(target_file, buffer.as_bytes())?;
    }

    // Rust

    Ok(())
}

fn run_formatter(cmd: &[String], source: &str) -> Result<String, Error> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    if cmd.is_empty() {
        return Err(Error::Formatter("Formatter command empty".to_owned()));
    }

    log::info!("Running formatter {:?}", cmd);
    let mut p = Command::new(cmd[0].clone())
        .args(&cmd[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    p.stdin.as_mut().unwrap().write_all(source.as_bytes())?;
    let output = p.wait_with_output().expect("failed to wait on child");

    if !output.status.success() {
        return Err(Error::Formatter(format!(
            "Formatter returned with non-zero exit code {:?}",
            output.status.code()
        )));
    }

    Ok(String::from_utf8(output.stdout).expect("Non-utf8 output from formatter"))
}
