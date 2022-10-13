extern crate core;

use clap::{Parser, Subcommand};

/// CLI for daimojo libraries
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the daimojo library
    #[arg(long,default_value="lib/linux_x64/libdaimojo.so")]
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
    Predict,
}

fn main() {
    match run() {
        Ok(0) => {}
        Ok(code) => {
            eprintln!("Exiting with code={code}");
            std::process::exit(code);
        }
        Err(e) => {
            eprintln!("ERROR: {e:?}");
            std::process::exit(1)
        }
    }
}

fn run() -> std::io::Result<i32> {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match cli.command {
        Commands::Show => {
            return show_pipeline(&cli.lib, &cli.mojo);
        }
        Commands::Predict => {
            todo!()
        }
    }
}

fn show_pipeline(lib: &str, mojo: &str) -> std::io::Result<i32> {
    println!("Opening library: '{lib}'");
    let lib = daimojo::DaiMojo::library(lib)?;
    println!("* library version is {}", lib.version());
    println!("Opening pipeline: '{mojo}'");
    let pipeline = lib.pipeline(mojo)?;
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
