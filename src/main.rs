use color_scheme_generator::{database, theme_calculation};
use std::{
    io::{stdin, IsTerminal, Read},
    path::PathBuf,
};

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(required = true, index = 1, value_parser = is_image)]
    image: PathBuf,
    #[arg(short, long, default_value_t = theme_calculation::Centrality::Prevalent)]
    centrality: theme_calculation::Centrality,
    #[arg(short, long, default_value_t = 5)]
    number_of_themes: u8,
}

fn is_image(input: &str) -> anyhow::Result<PathBuf> {
    let path = input.parse::<PathBuf>()?;
    let conn = database::DatabaseConnection::new(&PathBuf::from("/home/uwu/Downloads/temp.db"))?;
    match conn.select_color_theme_by_image_path(&path) {
        Ok(_) => Ok(path),
        Err(_) => {
            image::io::Reader::open(input)?
                .with_guessed_format()?
                .format()
                .ok_or(std::fmt::Error)?;
            Ok(path)
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args = if stdin().is_terminal() {
        Args::parse()
    } else {
        let mut input = String::new();
        let mut stdin = stdin().lock();
        while let Ok(x) = stdin.read_to_string(&mut input) {
            if x == 0 {
                break;
            }
        }
        let input = String::from(input.trim());
        let mut args = std::env::args().collect::<Vec<_>>();
        args.push(input);
        Args::parse_from(args.iter())
    };
    let conn = database::DatabaseConnection::new(&PathBuf::from("/home/uwu/Downloads/temp.db"))?;
    let json: String = match conn.select_color_theme_by_image_path(&args.image) {
        Ok(c) => 
            serde_json::to_string_pretty::<Vec<theme_calculation::ColorTheme>>(&c)?,
        Err(_) => {
	   let theme = theme_calculation::generate_color_theme(&args.image, args.centrality, args.number_of_themes);
		conn.insert_color_theme_record(&args.image, &theme)?;
	    serde_json::to_string_pretty::<Vec<theme_calculation::ColorTheme>>(&theme)?
	}
    };
    println!("{}", json);
    Ok(())
}
