#![deny(unused_extern_crates)]
#![warn(missing_docs)]

//! Quickly generate color schemes for waybar from an image.
//!
//! color_scheme_generator is a command line utility used to analyze images
//! and generate color themes from them given a path to an image.
//!
//! This command line utility behaves like a standard UNIX utility where the path to the image can be either piped in or sent a command line argument.
//!
//! The intended purpose of this application is to automatically create color themes for
//! Waybar, but it used for the bar in AwesomeWM or other applications to theme based on the on an image.
//! This utility has a cache for the image analysis. This means that once an image has been analyzed once, the result will be saved in the cache and when an image is analyzed again, the results will be returned instantly.
//!
//! # Usage Examples
//! ```bash
//! echo PATH_TO_IMAGE | color_scheme_generator
//! ```
//! ```bash
//! color_scheme_generator PATH_TO_IMAGE
//! ```
//!
//! # Output Formats
//! color_scheme_generator can output to 3 different output formats all of which give an RGB8 value in the form of "bar_color", "workspace_color" and "text_color":
//! 1. JSON
//! ```json
//! [{"bar_color":{"red":222,"green":186,"blue":189},"workspace_color":{"red":33,"green":69,"blue":66},"text_color":{"red":255,"green":255,"blue":255}}]
//! ```
//! 2. YAML
//! ```yaml
//! - bar_color:
//!     red: 222
//!     green: 186
//!     blue: 189
//!   workspace_color:
//!     red: 33
//!     green: 69
//!     blue: 66
//!   text_color:
//!     red: 255
//!     green: 255
//!     blue: 255
//! ```
//! 3. Text
//! ```bash
//! DEBABD,214542,FFFFFF
//! ```
//! The text output has the format of `BAR_COLOR,WORKSPACE_COLOR,TEXT_COLOR`.

use clap::Parser;
use color_scheme_generator::{
    common::{Centrality, Cli, Color, ColorThemes, OutputFormat, Wallpaper, APP_NAME},
    database, theme_calculation,
};
use log::{error, warn};
use std::io::{stdin, IsTerminal, Read};
use std::path::PathBuf;

fn is_image(path: &PathBuf) -> anyhow::Result<()> {
    image::ImageReader::open(path)?.with_guessed_format()?;
    Ok(())
}

fn is_default_color_theme_arguments(ct: &ColorThemes) -> bool {
    if ct.darker != 0
        || ct.lighter != 0
        || ct.complementary
        || ct.contrast
        || ct.hue_offset != 0
        || ct.triadic
        || ct.quadratic
        || ct.tetratic
        || ct.analogous
        || ct.split_complementary
        || ct.monochromatic != 0
        || ct.shades != 0
        || ct.tints != 0
        || ct.tones != 0
        || ct.blends != 0
    {
        return false;
    }
    true
}

/// Starting point of the application.
///
/// Check if program is in pipe, if so receive stdin and parse arguments and stdin.
/// Else, parse the arguments normally.
///
/// Creates cache inside of XDG_CACHE_HOME,
/// check if image is in cache, if so return theme,
/// else analyze the image and add it to cache.
fn main() -> anyhow::Result<()> {
    let mut args = if stdin().is_terminal() {
        Cli::parse()
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
        Cli::parse_from(args.iter())
    };

    stderrlog::new()
        .module(module_path!())
        .verbosity(!args.log_level)
        .init()
        .unwrap();

    if (args.color_themes.tetratic || args.color_themes.blends > 0)
        && args.centrality != Centrality::Prevalent
    {
        warn!("Incompatible centralitry argument. Switching to Prevalent.");
        args.centrality = Centrality::Prevalent
    }

    if is_default_color_theme_arguments(&args.color_themes) {
        args.color_themes.quadratic = true;
    }

    let xdg_dirs = xdg::BaseDirectories::with_prefix(APP_NAME)?;
    let cache_path = xdg_dirs.place_cache_file("cache.db")?;
    let conn = database::DatabaseConnection::new(&cache_path)?;

    let wallpaper = Wallpaper {
        path: args.image.clone(),
        centrality: args.centrality,
    };
    let color_themes = match conn.select_color_records(&wallpaper, &args.color_themes) {
        Ok(c) => c,
        Err(_) => {
            if is_image(&args.image).is_err() {
                error!("Inputted file is not an image");
                std::process::exit(1);
            }
            conn.insert_wallpaper_record(&wallpaper)?;
            conn.insert_color_themes_record(&args.color_themes, &wallpaper)?;
            let colors = crate::theme_calculation::generate_color_theme(&args)?;
            for color in &colors {
                conn.insert_color_record(color, &wallpaper, &args.color_themes)?;
            }
            colors
        }
    };

    let output: String = match args.serialization_format {
        OutputFormat::JSON => serde_json::to_string::<Vec<Color>>(&color_themes)?,
        OutputFormat::YAML => serde_yml::to_string::<Vec<Color>>(&color_themes)?,
        OutputFormat::TEXT => {
            let mut ret = String::new();
            color_themes
                .iter()
                .for_each(|c| ret += &format!("{},", c.color));
            let mut ret = String::from(&(&ret)[0..ret.len() - 2]);
            ret += "\n";
            ret
        }
    };
    println!("{}", output);
    Ok(())
}
