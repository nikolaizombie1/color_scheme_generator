#![deny(unused_extern_crates)]
#![warn(missing_docs)]
use rayon::prelude::*;
use crate::common::{RGB, Centrality, Color, GAMUT_CLI_NAME, Cli, ColorThemes};
use std::process::Command;
use which::which;



/// Get a [`Vec<ColorTheme>`] for an image based on the centrality and number of themes.
///
/// # Notes
/// The number_of_themes is ignored and set to 1 if the centrality is either [`Centrality::Average`] or [`Centrality::Median`].
/// This is due to the fact that for either of these centrality metrics, they give a single result and more results cannot be derived from them.
/// This is not the case for [`Centrality::Prevalent`] since a list of pixels can be generated using this method.
///
/// # Errors
///
/// If the path to the image is invalid (i.e does not exist or is not a valid image) this method will return an error. It is expected that the image
/// path is correct before using this method.
///
/// # Examples
/// ```
/// # use std::path::PathBuf;
/// # use color_scheme_generator::theme_calculation::Centrality;
/// # use color_scheme_generator::theme_calculation::generate_color_theme;
/// # let image_path = "test.png".parse::<PathBuf>().unwrap();
/// generate_color_theme(&image_path, Centrality::Prevalent, 10);
/// ```
pub fn generate_color_theme(
    args: &Cli
) -> anyhow::Result<Vec<Color>> {
    let pixels = image::ImageReader::open(&args.image)?
        .decode()?
        .to_rgb8()
        .pixels()
        .copied()
        .collect::<Vec<_>>();
    let bar_color = match args.centrality {
        Centrality::Average => vec![average_pixel(&pixels)],
        Centrality::Median => vec![median_pixel(&pixels)],
        Centrality::Prevalent => prevalent_pixel(&pixels, 2),
    };
    match args.centrality {
        Centrality::Average | Centrality::Median => Ok(call_gamut_cli(&args.color_themes, &bar_color[0], None)?),
        Centrality::Prevalent => Ok(call_gamut_cli(&args.color_themes, &bar_color[0], Some(&bar_color[1]))?),
    }
}


/// Get the average pixel from an image.
///
/// The average is the sum of each sub pixel divided by the total amount of pixels.
fn average_pixel(pixels: &[image::Rgb<u8>]) -> RGB {
    RGB {
        red: u8::try_from(
            pixels
                .par_iter()
                .map(|p| usize::from(p.0[0]))
                .sum::<usize>()
                / pixels.len(),
        )
        .unwrap(),
        green: u8::try_from(
            pixels
                .par_iter()
                .map(|p| usize::from(p.0[1]))
                .sum::<usize>()
                / pixels.len(),
        )
        .unwrap(),
        blue: u8::try_from(
            pixels
                .par_iter()
                .map(|p| usize::from(p.0[2]))
                .sum::<usize>()
                / pixels.len(),
        )
        .unwrap(),
    }
}


/// Get the median pixel from an image
///
/// The median is the middle value of each sub pixel inside of a sorted list.
fn median_pixel(pixels: &[image::Rgb<u8>]) -> RGB {
    RGB {
        red: median(&pixels.par_iter().map(|p| p.0[0]).collect::<Vec<_>>()),
        green: median(&pixels.par_iter().map(|p| p.0[1]).collect::<Vec<_>>()),
        blue: median(&pixels.par_iter().map(|p| p.0[2]).collect::<Vec<_>>()),
    }
}

/// Get the median value from a slice of [`u8`].
fn median(color_slice: &[u8]) -> u8 {
    if color_slice.len() % 2 == 0 {
        let left_middle =
            color_slice[(((color_slice.len() as f64) / (2.0)) - 1.0).floor() as usize];
        let right_middle = color_slice[((color_slice.len() as f64) / (2.0)).floor() as usize];
        (((right_middle as f64) + (left_middle as f64)) / 2.0) as u8
    } else {
        color_slice[(((color_slice.len() as f64) / 2.0) - 1.0) as usize]
    }
}

/// Get the pixels that appear the most times from an image.
///
/// # Note
/// Will return a [`Vec<ColorTheme>`], whose size will be either number_of_themes
/// or the amount of distinct rgb pixels in the image. The smaller of these two amounts
/// will be the size of the returned vector.
fn prevalent_pixel(pixels: &[image::Rgb<u8>], number_of_themes: u8) -> Vec<RGB> {
    let mut pixel_prevalence_count = std::collections::HashMap::new();
    for pixel in pixels.iter() {
        let count = pixel_prevalence_count.entry(pixel).or_insert(0);
        *count += 1;
    }
    let mut most_prevalent = pixel_prevalence_count
        .par_iter()
        .map(|x| (x.0, x.1))
        .collect::<Vec<_>>();
    most_prevalent.sort_by(|a,b| b.1.cmp(a.1));
    if most_prevalent.len() > number_of_themes as usize {
        most_prevalent[0..(number_of_themes as usize)]
            .par_iter()
            .map(|x| RGB {
                red: x.0[0],
                green: x.0[1],
                blue: x.0[2],
            })
            .collect::<Vec<_>>()
        
    } else {
        most_prevalent
            .par_iter()
            .map(|x| RGB {
                red: x.0[0],
                green: x.0[1],
                blue: x.0[2],
            })
            .collect::<Vec<_>>()
    }
}

fn call_gamut_cli(args: &ColorThemes, color1: &RGB, color2: Option<&RGB>) -> Result<Vec<Color>, anyhow::Error> {
    //which(GAMUT_CLI_NAME)?;
    let color2str = match color2 {
        Some(c) => c,
        None => &RGB{blue: 0, green: 0, red: 0},
    };
    let gamut_command = format!("gamut-cli {} -Color1 {color1} -Color2 {color2str}", args);
    let gamut_output = String::from_utf8(
	Command::new("bash")
	    .arg("-c").arg(&gamut_command).output()?.stdout)?.trim().to_owned();
    Ok(serde_json::from_str::<Vec<Color>>(&gamut_output)?)
}
