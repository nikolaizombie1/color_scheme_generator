#![deny(unused_extern_crates)]
#![warn(missing_docs)]
use crate::common::{Centrality, ColorThemeOption, Wallpaper, RGB};
use sqlite::Connection;
use sqlite::Row;
use std::path::PathBuf;
use std::str::FromStr;

/// Hold a sqlite database connection.
pub struct DatabaseConnection {
    connection: Connection,
}

impl DatabaseConnection {
    /// Create database cache file and connect to it.
    ///
    /// # Notes
    ///
    /// This method creates a sqlite database with three tables: wallpaper, color_themes, and RGB which represent the [`Wallpaper`], [`ColorThemeOption`], and [`RGB`] respectively.
    /// Every color_themes record must have a valid wallpaper record attached to it and every RGB record must have a valid wallpaper and color_themes record attached to it.
    ///
    /// # Errors
    ///
    /// If the database file cannot be created, albeit due to insufficient permissions or an invalid path, the method will throw an error.
    ///
    /// # Examples
    /// ```
    /// # use std::path::PathBuf;
    /// # use color_scheme_generator::database::DatabaseConnection;
    /// # let cache_path = ":memory:".parse::<PathBuf>().unwrap();
    /// let database_connection = DatabaseConnection::new(&cache_path).unwrap();
    /// ```
    pub fn new(path: &PathBuf) -> anyhow::Result<DatabaseConnection> {
        let conn = sqlite::open(path)?;
        let query = "
        CREATE TABLE IF NOT EXISTS wallpaper(path TEXT NOT NULL, centrality TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS color_themes(darker INTEGER NOT NULL, lighter INTEGER NOT NULL, complementary INTEGER NOT NULL, contrast INTEGER NOT NULL, hueOffset INTEGER NOT NULL, triadic INTEGER NOT NULL, quadratic INTEGER NOT NULL, tetratic INTEGER NOT NULL, analogous INTEGER NOT NULL, splitComplementary INTEGER NOT NULL, monochromatic INTEGER NOT NULL, shades INTEGER NOT NULL, tints INTEGER NOT NULL, tones INTEGER NOT NULL, blends INTEGER NOT NULL, wallpaper INTEGER NOT NULL, FOREIGN KEY(wallpaper) REFERENCES wallpaper(ROWID));
        CREATE TABLE IF NOT EXISTS RGB(RGB TEXT NOT NULL, wallpaper INTEGER NOT NULL, color_themes INTEGER NOT NULL, FOREIGN KEY(wallpaper) REFERENCES wallpaper(ROWID), FOREIGN KEY(color_themes) REFERENCES color_themes(ROWID));
        ";
        conn.execute(query)?;
        Ok(DatabaseConnection { connection: conn })
    }

    /// Insert a wallpaper record into the database
    ///
    /// # Errors
    ///
    /// Should only error only if the path cannot be converted to a [`&str`].
    ///
    /// # Examples
    /// ```
    /// # use std::path::PathBuf;
    /// # use color_scheme_generator::database::DatabaseConnection;
    /// # use color_scheme_generator::common::{Wallpaper, Centrality};
    /// # let cache_path = ":memory:".parse::<PathBuf>().unwrap();
    /// let database_connection = DatabaseConnection::new(&cache_path).unwrap();
    /// # let wallpaper = Wallpaper {path : "text".parse::<PathBuf>().unwrap(), centrality: Centrality::Prevalent};
    /// database_connection.insert_wallpaper_record(&wallpaper).unwrap();
    /// ```
    pub fn insert_wallpaper_record(&self, wallpaper: &Wallpaper) -> anyhow::Result<()> {
        let query = format!(
            "INSERT INTO wallpaper(path, centrality) VALUES ('{}', '{}')",
            wallpaper.path.to_str().ok_or(std::fmt::Error)?,
            wallpaper.centrality
        );
        self.connection.execute(query)?;
        Ok(())
    }

    /// Select a wallpaper record  from the database.
    ///
    /// # Errors
    ///
    /// Will error if the record is not found in the database.
    ///
    /// # Examples
    /// ```
    /// # use std::path::PathBuf;
    /// # use color_scheme_generator::database::DatabaseConnection;
    /// # use color_scheme_generator::common::{Wallpaper, Centrality};
    /// # let cache_path = ":memory:".parse::<PathBuf>().unwrap();
    /// let database_connection = DatabaseConnection::new(&cache_path).unwrap();
    /// # let wallpaper = Wallpaper {path : "text".parse::<PathBuf>().unwrap(), centrality: Centrality::Prevalent};
    /// # database_connection.insert_wallpaper_record(&wallpaper).unwrap();
    /// # let wallpaper_record = database_connection.select_wallpaper_record(&wallpaper).unwrap();
    /// ```
    pub fn select_wallpaper_record(
        &self,
        wallpaper: &Wallpaper,
    ) -> anyhow::Result<(Wallpaper, i64)> {
        let query = format!(
            "SELECT path, centrality, ROWID as PK FROM wallpaper where path = '{}' AND centrality = '{}'",
            wallpaper.path.to_str().ok_or(std::fmt::Error)?,
            wallpaper.centrality
        );
        let row = self
            .connection
            .prepare(&query)?
            .into_iter()
            .map(|r| r.unwrap())
            .collect::<Vec<_>>();
        let path = self
            .get_database_column::<&str>(&row, "path")
            .iter()
            .map(PathBuf::from)
            .collect::<Vec<_>>()
            .first()
            .ok_or(std::fmt::Error)?
            .to_owned();
        let centrality = self.get_database_column::<&str>(&row, "centrality")?;
        let centrality = Centrality::from_str(centrality)?;
        let rowid = row
            .iter()
            .map(|r| r.read::<i64, _>("PK"))
            .collect::<Vec<_>>()
            .first()
            .ok_or(std::fmt::Error)?
            .to_owned();
        Ok((Wallpaper { path, centrality }, rowid))
    }

    /// Insert a color_theme record into the database.
    ///
    /// # Notes
    /// The [`Wallpaper`] must be inserted into the database before a [`ColorThemeOption`] record can be successfully inserted since the [`Wallpaper`] ROWID is referenced by a [`ColorThemeOption`] record.
    /// The [`ColorThemeOption`] struct must have only 1 field that is not a default value. call_gamut_cli depends on this struct being constructed correctly. Clap and main take care of this normally but special care is needed when interacting with this struct directly.
    ///
    /// # Errors
    /// Will error if a [`Wallpaper`] record cannot be found inside the database.
    ///
    /// # Examples
    /// ```
    /// # use std::path::PathBuf;
    /// # use color_scheme_generator::database::DatabaseConnection;
    /// # use color_scheme_generator::common::{Wallpaper, Centrality, ColorThemeOption};
    /// # let cache_path = ":memory:".parse::<PathBuf>().unwrap();
    /// let database_connection = DatabaseConnection::new(&cache_path).unwrap();
    /// # let wallpaper = Wallpaper {path : "text".parse::<PathBuf>().unwrap(), centrality: Centrality::Prevalent};
    /// # database_connection.insert_wallpaper_record(&wallpaper).unwrap();
    /// # let color_themes = ColorThemeOption {
    /// #   darker: 0,
    /// #   lighter: 0,
    /// #   complementary: false,
    /// #   contrast: false,
    /// #   hue_offset: 0,
    /// #   triadic: false,
    /// #   quadratic: true,
    /// #   tetratic: false,
    /// #   analogous: false,
    /// #   split_complementary: false,
    /// #   monochromatic: 0,
    /// #   shades: 0,
    /// #   tints: 0,
    /// #   tones: 0,
    /// #   blends: 0,
    /// # };
    /// database_connection.insert_color_themes_record(&color_themes, &wallpaper).unwrap();
    /// ```
    pub fn insert_color_themes_record(
        &self,
        ct: &ColorThemeOption,
        wallpaper: &Wallpaper,
    ) -> anyhow::Result<()> {
        let query = format!(
            "INSERT INTO color_themes
                                        (darker, 
                                        lighter,
                                        complementary,
                                        contrast,
                                        hueOffset,
                                        triadic,
                                        quadratic,
                                        tetratic,
                                        analogous,
                                        splitComplementary,
                                        monochromatic,
                                        shades,
                                        tints,
                                        tones,
                                        blends,
                                        wallpaper) VALUES
                                        ({},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{})",
            ct.darker,
            ct.lighter,
            ct.complementary,
            ct.contrast,
            ct.hue_offset,
            ct.triadic,
            ct.quadratic,
            ct.tetratic,
            ct.analogous,
            ct.split_complementary,
            ct.monochromatic,
            ct.shades,
            ct.tints,
            ct.tones,
            ct.blends,
            self.select_wallpaper_record(wallpaper)?.1
        );
        self.connection.execute(query)?;
        Ok(())
    }

    /// Select [`ColorThemeOption`] record from the database.
    ///
    /// # Notes
    /// The [`Wallpaper`] must be inserted into the database before a [`ColorThemeOption`] record can be successfully selected since the [`Wallpaper`] ROWID is referenced by a [`ColorThemeOption`] record.
    ///
    /// # Errors
    /// Will error if either the [`Wallpaper`] record is not found or if the [`ColorThemeOption`] record is not found.
    ///
    /// # Examples
    /// ```
    /// # use std::path::PathBuf;
    /// # use color_scheme_generator::database::DatabaseConnection;
    /// # use color_scheme_generator::common::{Wallpaper, Centrality, ColorThemeOption};
    /// # let cache_path = ":memory:".parse::<PathBuf>().unwrap();
    /// let database_connection = DatabaseConnection::new(&cache_path).unwrap();
    /// # let wallpaper = Wallpaper {path : "text".parse::<PathBuf>().unwrap(), centrality: Centrality::Prevalent};
    /// # database_connection.insert_wallpaper_record(&wallpaper).unwrap();
    /// # let color_themes = ColorThemeOption {
    /// #   darker: 0,
    /// #   lighter: 0,
    /// #   complementary: false,
    /// #   contrast: false,
    /// #   hue_offset: 0,
    /// #   triadic: false,
    /// #   quadratic: true,
    /// #   tetratic: false,
    /// #   analogous: false,
    /// #   split_complementary: false,
    /// #   monochromatic: 0,
    /// #   shades: 0,
    /// #   tints: 0,
    /// #   tones: 0,
    /// #   blends: 0,
    /// # };
    /// # database_connection.insert_color_themes_record(&color_themes, &wallpaper).unwrap();
    /// database_connection.select_color_themes_record(&color_themes, &wallpaper).unwrap();
    /// ```
    pub fn select_color_themes_record(
        &self,
        ct: &ColorThemeOption,
        wallpaper: &Wallpaper,
    ) -> anyhow::Result<(ColorThemeOption, i64)> {
        let query = format!(
            "SELECT darker, lighter, complementary, contrast, hueOffset, triadic, quadratic, tetratic, analogous, splitComplementary, monochromatic, shades, tints, tones, blends, ROWID as PK FROM color_themes WHERE darker = {} AND 
                                        lighter = {} AND
                                        complementary = {} AND
                                        contrast = {} AND
                                        hueOffset = {} AND
                                        triadic = {} AND
                                        quadratic = {} AND
                                        tetratic = {} AND
                                        analogous = {} AND
                                        splitComplementary = {} AND
                                        monochromatic = {} AND
                                        shades = {} AND
                                        tints = {} AND
                                        tones = {} AND
                                        blends = {} AND
                                        wallpaper = {}",
            ct.darker,
            ct.lighter,
            ct.complementary,
            ct.contrast,
            ct.hue_offset,
            ct.triadic,
            ct.quadratic,
            ct.tetratic,
            ct.analogous,
            ct.split_complementary,
            ct.monochromatic,
            ct.shades,
            ct.tints,
            ct.tones,
            ct.blends,
            self.select_wallpaper_record(wallpaper)?.1
        );
        let row = self
            .connection
            .prepare(&query)?
            .into_iter()
            .map(|r| r.unwrap())
            .collect::<Vec<_>>();
        let color_themes = ColorThemeOption {
            darker: u8::try_from(self.get_database_column::<i64>(&row, "darker")?)?,
            lighter: u8::try_from(self.get_database_column::<i64>(&row, "lighter")?)?,
            complementary: i64_to_bool(self.get_database_column(&row, "complementary")?),
            contrast: i64_to_bool(self.get_database_column(&row, "contrast")?),
            hue_offset: u16::try_from(self.get_database_column::<i64>(&row, "hueOffset")?)?,
            triadic: i64_to_bool(self.get_database_column(&row, "triadic")?),
            quadratic: i64_to_bool(self.get_database_column(&row, "quadratic")?),
            tetratic: i64_to_bool(self.get_database_column(&row, "tetratic")?),
            analogous: i64_to_bool(self.get_database_column(&row, "analogous")?),
            split_complementary: i64_to_bool(self.get_database_column(&row, "splitComplementary")?),
            monochromatic: u8::try_from(self.get_database_column::<i64>(&row, "lighter")?)?,
            shades: u8::try_from(self.get_database_column::<i64>(&row, "shades")?)?,
            tints: u8::try_from(self.get_database_column::<i64>(&row, "tints")?)?,
            tones: u8::try_from(self.get_database_column::<i64>(&row, "tones")?)?,
            blends: u8::try_from(self.get_database_column::<i64>(&row, "blends")?)?,
        };
        let rowid = self.get_database_column::<i64>(&row, "PK")?;
        Ok((color_themes, rowid))
    }

    /// Insert [`RGB`] record into the database
    ///
    /// # Notes
    /// Both [`ColorThemeOption`] and [`Wallpaper`] records have to be inserted into the database before successfully inserting a [`RGB`] record.
    /// 
    /// # Errors
    /// Will throw an error if either [`Wallpaper`] or [`ColorThemeOption`] is not found in the database.
    /// 
    /// # Examples
    /// ```
    /// # use std::path::PathBuf;
    /// # use color_scheme_generator::database::DatabaseConnection;
    /// # use color_scheme_generator::common::{Wallpaper, Centrality, ColorThemeOption, RGB};
    /// # let cache_path = ":memory:".parse::<PathBuf>().unwrap();
    /// let database_connection = DatabaseConnection::new(&cache_path).unwrap();
    /// # let wallpaper = Wallpaper {path : "text".parse::<PathBuf>().unwrap(), centrality: Centrality::Prevalent};
    /// # database_connection.insert_wallpaper_record(&wallpaper).unwrap();
    /// # let color_themes = ColorThemeOption {
    /// #   darker: 0,
    /// #   lighter: 0,
    /// #   complementary: false,
    /// #   contrast: false,
    /// #   hue_offset: 0,
    /// #   triadic: false,
    /// #   quadratic: true,
    /// #   tetratic: false,
    /// #   analogous: false,
    /// #   split_complementary: false,
    /// #   monochromatic: 0,
    /// #   shades: 0,
    /// #   tints: 0,
    /// #   tones: 0,
    /// #   blends: 0,
    /// # };
    /// # database_connection.insert_color_themes_record(&color_themes, &wallpaper).unwrap();
    /// # let RGB = RGB {red: 255, green: 0, blue: 0};
    /// database_connection.insert_rgb_record(&RGB, &wallpaper, &color_themes).unwrap();
    /// ```
    pub fn insert_rgb_record(
        &self,
        rgb: &RGB,
        wallpaper: &Wallpaper,
        ct: &ColorThemeOption,
    ) -> anyhow::Result<()> {
        let query = format!(
            "INSERT INTO RGB (RGB, wallpaper, color_themes) VALUES ('{}', {}, {})",
            rgb,
            self.select_wallpaper_record(wallpaper)?.1,
            self.select_color_themes_record(ct, wallpaper)?.1
        );
        self.connection.execute(query)?;
        Ok(())
    }

    /// Select  [`RGB`] record in from the database.
    /// 
    /// # Notes
    /// A [`Wallpaper`] and [`ColorThemeOption`] must be inserted into the database before a [`RGB`] record can be successfully selected since the [`Wallpaper`] ROWID and [`ColorThemeOption`] ROWID is referenced by a [`RGB`] record.
    /// 
    /// # Errors
    /// Will throw an error if:
    /// - [`Wallpaper`] record is not found in the database.
    /// - [`ColorThemeOption`] record is not found in the database.
    /// - [`RGB`] record is not found in the database.
    /// # Examples
    /// ```
    /// # use std::path::PathBuf;
    /// # use color_scheme_generator::database::DatabaseConnection;
    /// # use color_scheme_generator::common::{Wallpaper, Centrality, ColorThemeOption, RGB};
    /// # let cache_path = ":memory:".parse::<PathBuf>().unwrap();
    /// let database_connection = DatabaseConnection::new(&cache_path).unwrap();
    /// # let wallpaper = Wallpaper {path : "text".parse::<PathBuf>().unwrap(), centrality: Centrality::Prevalent};
    /// # database_connection.insert_wallpaper_record(&wallpaper).unwrap();
    /// # let color_themes = ColorThemeOption {
    /// #   darker: 0,
    /// #   lighter: 0,
    /// #   complementary: false,
    /// #   contrast: false,
    /// #   hue_offset: 0,
    /// #   triadic: false,
    /// #   quadratic: true,
    /// #   tetratic: false,
    /// #   analogous: false,
    /// #   split_complementary: false,
    /// #   monochromatic: 0,
    /// #   shades: 0,
    /// #   tints: 0,
    /// #   tones: 0,
    /// #   blends: 0,
    /// # };
    /// # database_connection.insert_color_themes_record(&color_themes, &wallpaper).unwrap();
    /// # let RGB = RGB {red: 255, green: 0, blue: 0};
    /// # database_connection.insert_rgb_record(&RGB, &wallpaper, &color_themes).unwrap();
    /// database_connection.select_rgb_records(&wallpaper, &color_themes).unwrap();
    pub fn select_rgb_records(
        &self,
        wallpaper: &Wallpaper,
        ct: &ColorThemeOption,
    ) -> anyhow::Result<Vec<RGB>> {
        let query = format!(
            "SELECT RGB FROM RGB where wallpaper = {} AND color_themes = {} ORDER BY ROWID;",
            self.select_wallpaper_record(wallpaper)?.1,
            self.select_color_themes_record(ct, wallpaper)?.1
        );
        let colors = self
            .connection
            .prepare(&query)?
            .into_iter()
            .map(|r| r.unwrap())
            .collect::<Vec<_>>();
        let colors = colors
            .iter()
            .map(|r| r.read::<&str, _>("RGB"))
            .map(|r| String::from_str(r).unwrap())
            .map(|s| RGB::from_str(&s).unwrap())
            .collect::<Vec<_>>();
        Ok(colors)
    }

    fn get_database_column<'a, T>(&'a self, row: &'a [Row], column: &str) -> anyhow::Result<T>
    where
        T: TryFrom<&'a sqlite::Value, Error = sqlite::Error>,
        T: Clone,
        T: Copy,
    {
        let binding = row
            .iter()
            .map(|r| r.read::<T, _>(column))
            .collect::<Vec<_>>();
        let x = binding.first().ok_or(std::fmt::Error)?;
        Ok(*x)
    }
}

fn i64_to_bool(num: i64) -> bool {
    !matches!(num, 0)
}
