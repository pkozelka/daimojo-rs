extern crate core;

use std::path::PathBuf;
use std::process::ExitCode;
use std::str::FromStr;
use clap::{ArgAction, Parser, Subcommand};
use log::LevelFilter;
use daimojo::MojoPipeline;

/// CLI for daimojo libraries
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Logging in verbose mode (-v = DEBUG, -vv = TRACE)
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,
    /// Logging in silent mode (-s = WARN, -ss = ERROR, -sss = OFF)
    #[arg(short, long, action = ArgAction::Count)]
    silent: u8,

    /// Path to the daimojo library
    #[arg(long,default_value="libdaimojo.so")]
    lib: String,

    /// Path to the pipeline
    #[arg(long,value_name="PIPELINE",default_value="pipeline.mojo")]
    mojo: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show some data about the pipeline
    Show,
    /// Run prediction
    Predict {
        /// Set batch size. For 0, it is determined automatically
        #[arg(long="batch",default_value="0")]
        batch_size: usize,
        #[arg(long="out")]
        output: Option<String>,
        //TODO later, this will probably be Vec<String>
        input: Option<String>,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(0) => ExitCode::SUCCESS,
        Ok(code) => {
            //TODO: I don't know why logging doesn't work here...
            // log::error!("Exiting with code={code}");
            eprintln!("Exiting with code={code}");
            ExitCode::from(code)
        }
        Err(e) => {
            //TODO: I don't know why logging doesn't work here...
            eprintln!("ERROR: {e:?}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> std::io::Result<u8> {
    let cli = Cli::parse();

    // library path must always contain a directory so we canonicalize it - it's the easiest way to get its absolute path
    let lib = PathBuf::from_str(&cli.lib).unwrap()
        .canonicalize()?
        .to_string_lossy()
        .to_string();
    // setup logger
    let level: i8 = 3 as i8 + cli.verbose as i8 - cli.silent as i8;
    let level = match level {
        i8::MIN..=0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        5..=i8::MAX => LevelFilter::Trace,
    };
    pretty_env_logger::formatted_timed_builder()
        .format_timestamp_millis()
        .filter_level(level.into())
        .init();
    // run subcommand
    match cli.command {
        Commands::Show => {
            return show_pipeline(&lib, &cli.mojo)
        }
        Commands::Predict {output, input, batch_size} => {
            let pipeline = open_pipeline(&cli.lib, &cli.mojo)?;
            cmd_predict::cmd_predict(&pipeline, output, input, batch_size)
        }
    }
}

fn show_pipeline(lib: &str, mojo: &str) -> std::io::Result<u8> {
    let pipeline = open_pipeline(lib, mojo)?;
    println!("* UUID: {}", pipeline.uuid());
    println!("* Time created: {}", pipeline.time_created());
    println!("* Missing values: {}", pipeline.missing_values().join(", "));
    let inputs = pipeline.inputs();
    println!("Input features[{}]:", inputs.len());
    for (col_name, col_type) in inputs {
        println!("* '{col_name}': {col_type:?}");
    }
    let outputs = pipeline.outputs();
    println!("Output columns[{}]:", outputs.len());
    for (col_name, col_type) in outputs {
        println!("* '{col_name}': {col_type:?}");
    }
    Ok(0)
}

fn open_pipeline(lib: &str, mojo: &str) -> std::io::Result<MojoPipeline> {
    log::debug!("Opening library: '{lib}'");
    let lib = daimojo::DaiMojo::library(lib)?;
    println!("Library's daimojo version is {}", lib.version());
    log::debug!("Opening pipeline: '{mojo}'");
    let pipeline = lib.pipeline(mojo)?;
    Ok(pipeline)
}

mod cmd_predict;
