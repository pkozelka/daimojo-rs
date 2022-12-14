extern crate core;

use std::process::ExitCode;
use clap::{ArgAction, Parser, Subcommand};
use log::LevelFilter;
use daimojo::daimojo_library::{DaiMojoLibrary, MOJO_Transform_Flags, RawColumnMeta, RawFlags, RawModel, RawPipeline};

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

fn run() -> anyhow::Result<u8> {
    let cli = Cli::parse();

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
            let lib = load_library(&cli.lib)?;
            return Ok(show_pipeline(&lib, &cli.mojo)?)
        }
        Commands::Predict {output, input, batch_size} => {
            let lib = load_library(&cli.lib)?;
            let model = load_model(&lib, &cli.mojo)?;
            let pipeline = RawPipeline::new(&model, MOJO_Transform_Flags::PREDICT as RawFlags)?;
            Ok(cmd_predict::cmd_predict(&pipeline, output, input, batch_size)?)
        }
    }
}

fn show_pipeline(lib: &DaiMojoLibrary, mojo: &str) -> anyhow::Result<u8> {
    let model = load_model(lib, mojo)?;
    println!("* UUID: {}", model.uuid());
    println!("* Time created: {}", model.time_created_utc());
    println!("* Missing values: {}", model.missing_values().join(", "));
    let features = model.features();
    println!("Input features[{}]:", features.len());
    for RawColumnMeta { name, column_type} in features {
        println!("* '{name}': {column_type:?}");
    }
    let pipeline = RawPipeline::new(&model, MOJO_Transform_Flags::PREDICT as RawFlags)?; //TODO
    let outputs = pipeline.outputs();
    println!("Output columns[{}]:", outputs.len());
    for RawColumnMeta { name, column_type} in outputs {
        println!("* '{name}': {column_type:?}");
    }
    Ok(0)
}

fn load_library(lib: &str) -> daimojo::Result<DaiMojoLibrary> {
    log::debug!("Loading library: '{lib}'");
    let lib = DaiMojoLibrary::load(lib)?;
    println!("Library's daimojo version is {}", lib.version());
    Ok(lib)
}

fn load_model<'a>(lib: &'a DaiMojoLibrary, mojo: &str) -> daimojo::Result<RawModel<'a>> {
    log::debug!("Loading mojo: '{mojo}'");
    let model = RawModel::load(&lib, mojo, ".")?;
    Ok(model)
}

mod cmd_predict;
