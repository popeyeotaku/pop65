use std::{error::Error, fs};

use clap::Parser;
use pop65::{assemble, dump_symtab};

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let (bytes, symtab) = assemble(pop65::from_file(&cli.source)?)?;
    fs::write(cli.output, bytes)?;
    if let Some(sympath) = cli.symbols {
        let symstr = dump_symtab(symtab);
        fs::write(sympath, symstr)?;
    }
    Ok(())
}

#[derive(Parser)]
#[command(version)]
struct Cli {
    source: String,

    #[arg(short, long)]
    output: String,

    #[arg(short, long)]
    symbols: Option<String>,
}
