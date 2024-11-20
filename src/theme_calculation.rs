#![deny(unused_extern_crates)]
#![warn(missing_docs)]
use clap::ValueEnum;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf};

/// Measures of centrality to generate ColorTheme.
#[derive(PartialEq, Copy, Clone, ValueEnum)]
pub enum Centrality {
    /// Takes the sum of the pixels and divides by the amount of pixels in an image.
    Average,
    /// Sort the pixels and takes the middlemost pixel in an image.
    Median,
    /// Get the most repeating pixels in an image.
    Prevalent,
}

impl Display for Centrality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Centrality::Average => write!(f, "average"),
            Centrality::Median => write!(f, "median"),
            Centrality::Prevalent => write!(f, "prevalent"),
        }
    }
}

/// Struct representation for the [`image::Rgb<u8>`] type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RGB {
    /// Red RGB sub-pixel.
    pub red: u8,
    /// Green RGB sub-pixel.
    pub green: u8,
    /// Blue RGB sub-pixel.
    pub blue: u8,
}

/// Representation for Sway Waybar color theme.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ColorTheme {
    /// The color determined by the [`Centrality`] metric.
    pub bar_color: RGB,
    /// The compliment of the [`ColorTheme::bar_color`].
    pub workspace_color: RGB,
    /// Either black or white depending on the [`ColorTheme::workspace_color`]
    pub text_color: RGB,
}

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
    path: &PathBuf,
    centrality: Centrality,
    number_of_themes: u8,
) -> anyhow::Result<Vec<ColorTheme>> {
    let pixels = image::ImageReader::open(path)?
        .decode()?
        .to_rgb8()
        .pixels()
        .copied()
        .collect::<Vec<_>>();
    let bar_color = match centrality {
        Centrality::Average => vec![average_pixel(&pixels)],
        Centrality::Median => vec![median_pixel(&pixels)],
        Centrality::Prevalent => prevalent_pixel(&pixels, number_of_themes),
    };
    match centrality {
        Centrality::Average | Centrality::Median => Ok(vec![(calculate_color_theme(&bar_color[0]))]),
        Centrality::Prevalent => Ok(bar_color
            .par_iter()
            .map(calculate_color_theme)
            .collect::<Vec<_>>()),
    }
}


/// Generate the [`ColorTheme::workspace_color`] and Generate the [`ColorTheme::text_color`]
/// For a given [`RGB`] value.
///
/// Examples
///
/// ```
/// use color_scheme_generator::theme_calculation::{RGB, calculate_color_theme, ColorTheme}; 
/// let pixel = RGB{red: 0, green: 0, blue: 0};
/// let result = calculate_color_theme(&pixel);
/// let expected = ColorTheme{bar_color: RGB{red: 0, green: 0, blue: 0}, workspace_color: RGB{red: 255, green: 255, blue: 255}, text_color: RGB{red: 0, green: 0, blue: 0}};
/// assert_eq!(expected, result);
/// ```
pub fn calculate_color_theme(rgb: &RGB) -> ColorTheme {
    let workspace_color = complementary_color(rgb);
    let text_color =
        if workspace_color.red > 128 && workspace_color.blue > 128 && workspace_color.green > 128 {
            RGB {
                red: 0,
                green: 0,
                blue: 0,
            }
        } else {
            RGB {
                red: 255,
                green: 255,
                blue: 255,
            }
        };
    ColorTheme {
        bar_color: rgb.clone(),
        workspace_color,
        text_color,
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

/// Get the compliment of an RGB pixel.
///
/// # Note
/// The formula used was 255 - pixel_value.
fn complementary_color(rgb: &RGB) -> RGB {
    RGB {
        red: 255 - rgb.red,
        green: 255 - rgb.green,
        blue: 255 - rgb.blue,
    }
}
