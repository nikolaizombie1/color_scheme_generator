use anyhow::Result;
use std::{
    io::{stdin, IsTerminal, Read}, path::PathBuf
};

use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(index = 1)]
    image: PathBuf,
}


fn main() -> Result<()> {
    let args = if stdin().is_terminal() {
	print!("Entered terminal branch");
	Args::parse()
    }
    else {
	let mut input = String::new();
    let mut stdin = stdin().lock();
    while let Ok(x) = stdin.read_to_string(&mut input) {
        if x == 0 {break;}
    }

	let input = shellwords::split(&input)?;
	Args::parse_from(input.iter())
    };
    println!("{}", args.image.as_path().to_str().unwrap());
    Ok(())
}
