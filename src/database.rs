#![deny(unused_extern_crates)]
#![warn(missing_docs)]
use crate::common::{Centrality, Color, ColorThemes, Wallpaper};
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
    /// This method creates a sqlite database with three tables: wallpaper, color_themes, and color which represent the [`Wallpaper`], [`ColorThemes`], and [`Color`] repectively.
    /// Every color_themes record must have a valid wallpaper record attached to it and every color record must have a valid wallpaper and color_themes record attached to it.
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
        CREATE TABLE IF NOT EXISTS color(color TEXT NOT NULL, wallpaper INTEGER NOT NULL, color_themes INTEGER NOT NULL, FOREIGN KEY(wallpaper) REFERENCES wallpaper(ROWID), FOREIGN KEY(color_themes) REFERENCES color_themes(ROWID));
        ";
        conn.execute(query)?;
        Ok(DatabaseConnection { connection: conn })
    }
    pub fn insert_wallpaper_record(&self, wallpaper: &Wallpaper) -> anyhow::Result<()> {
        let query = format!(
            "INSERT INTO wallpaper(path, centrality) VALUES ('{}', '{}')",
            wallpaper.path.to_str().ok_or(std::fmt::Error)?,
            wallpaper.centrality
        );
        self.connection.execute(query)?;
        Ok(())
    }

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

    pub fn insert_color_themes_record(
        &self,
        args: &ColorThemes,
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
            args.darker,
            args.lighter,
            args.complementary,
            args.contrast,
            args.hue_offset,
            args.triadic,
            args.quadratic,
            args.tetratic,
            args.analogous,
            args.split_complementary,
            args.monochromatic,
            args.shades,
            args.tints,
            args.tones,
            args.blends,
            self.select_wallpaper_record(wallpaper)?.1
        );
        self.connection.execute(query)?;
        Ok(())
    }
    pub fn select_color_theme_record(
        &self,
        ct: &ColorThemes,
        wallpaper: &Wallpaper,
    ) -> anyhow::Result<(ColorThemes, i64)> {
        let wallpaper_rowid = match self.select_wallpaper_record(wallpaper) {
            Ok(i) => i.1,
            Err(e) => {
                return Err(e);
            }
        };
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
            wallpaper_rowid
        );
        let row = self
            .connection
            .prepare(&query)?
            .into_iter()
            .map(|r| r.unwrap())
            .collect::<Vec<_>>();
        let color_themes = ColorThemes {
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

    pub fn insert_color_record(
        &self,
        color: &Color,
        wallpaper: &Wallpaper,
        ct: &ColorThemes,
    ) -> anyhow::Result<()> {
        let query = format!(
            "INSERT INTO color (color, wallpaper, color_themes) VALUES ('{}', {}, {})",
            color.color,
            self.select_wallpaper_record(wallpaper)?.1,
            self.select_color_theme_record(ct, wallpaper)?.1
        );
        self.connection.execute(query)?;
        Ok(())
    }

    pub fn select_color_records(
        &self,
        wallpaper: &Wallpaper,
        ct: &ColorThemes,
    ) -> anyhow::Result<Vec<Color>> {
        let query = format!(
            "SELECT color FROM color where wallpaper = {} AND color_themes = {} ORDER BY ROWID;",
            self.select_wallpaper_record(wallpaper)?.1,
            self.select_color_theme_record(ct, wallpaper)?.1
        );
        let colors = self
            .connection
            .prepare(&query)?
            .into_iter()
            .map(|r| r.unwrap())
            .collect::<Vec<_>>();
        let colors = colors
            .iter()
            .map(|r| r.read::<&str, _>("color"))
            .map(|r| String::from_str(r).unwrap())
            .map(|s| Color { color: s })
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
