use std::{error::Error, fs};

use clap::Parser;
use pop65::assemble;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let info = assemble(pop65::from_file(&cli.source)?, cli.list_file.is_some())?;
    if let Some(outpath) = cli.output {
        fs::write(outpath, &info.bytes)?;
    }
    if let Some(sympath) = cli.symbol_file {
        let symstr = info.dump_symtab();
        fs::write(sympath, symstr)?;
    }
    if let Some(dbgpath) = cli.debug_file {
        fs::write(dbgpath, &info.debug_str)?;
    }
    if let Some(listpath) = cli.list_file {
        fs::write(listpath, info.listing.unwrap())?;
    }
    Ok(())
}

#[derive(Parser)]
#[command(version)]
struct Cli {
    source: String,

    #[arg(short, long)]
    output: Option<String>,

    #[arg(short, long)]
    symbol_file: Option<String>,

    #[arg(short, long)]
    debug_file: Option<String>,

    #[arg(short, long)]
    list_file: Option<String>,
}
