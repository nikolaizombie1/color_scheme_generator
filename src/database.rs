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
    pub fn new(path: &PathBuf) -> anyhow::Result<DatabaseConnection> {
        let conn = sqlite::open(path)?;
        let query = "
        CREATE TABLE IF NOT EXISTS wallpaper(path TEXT NOT NULL, centrality TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS color_themes(darker INTEGER NOT NULL, 
                                        lighter INTEGER NOT NULL,
                                        complementary INTEGER NOT NULL,
                                        contrast INTEGER NOT NULL,
                                        hueOffset INTEGER NOT NULL,
                                        triadic INTEGER NOT NULL,
                                        quadratic INTEGER NOT NULL,
                                        tetratic INTEGER NOT NULL,
                                        analogous INTEGER NOT NULL,
                                        splitComplementary INTEGER NOT NULL,
                                        monochromatic INTEGER NOT NULL,
                                        shades INTEGER NOT NULL,
                                        tints INTEGER NOT NULL,
                                        tones INTEGER NOT NULL,
                                        blends INTEGER NOT NULL,
                                        wallpaper INTEGER NOT NULL,
                                        FOREIGN KEY(wallpaper) REFERENCES wallpaper(ROWID));
        CREATE TABLE IF NOT EXISTS color(color TEXT NOT NULL, wallpaper INTEGER NOT NULL, FOREIGN KEY(wallpaper) REFERENCES wallpaper(ROWID));
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
        let centrality = self
            .get_database_column::<&str>(&row, "centrality")?;
        let centrality = Centrality::from_str(centrality)?;
        let rowid = row.iter();
        let rowid = rowid.map(|r| r.read::<i64, _>("PK"));
        let rowid = rowid.collect::<Vec<_>>();
        let rowid = rowid.first();
        let rowid =  rowid.ok_or(std::fmt::Error)?;
        let rowid = rowid.to_owned();
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
    pub fn is_color_theme_equal(
        &self,
        ct: &ColorThemes,
        wallpaper: &Wallpaper,
    ) -> anyhow::Result<bool> {
        let query = format!(
            "SELECT COUNT(1) as c FROM color_theme WHERE darker = {} AND 
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
        let count = self
            .get_database_column::<i64>(&row, "c")?;
        if count == 0 {
            return Ok(false);
        }
        Ok(true)
    }

    pub fn insert_color_record(&self, color: &Color, wallpaper: &Wallpaper) -> anyhow::Result<()> {
        let query = format!(
            "INSERT INTO color (color, wallpaper) VALUES ('{}', {})",
            color.color,
            self.select_wallpaper_record(wallpaper)?.1
        );
        self.connection.execute(query)?;
        Ok(())
    }

    pub fn select_color_records(&self, wallpaper: &Wallpaper) -> anyhow::Result<Vec<Color>> {
        let query = format!(
            "SELECT color FROM color where wallpaper = {} ORDER BY ROWID;",
            self.select_wallpaper_record(wallpaper)?.1
        );
        let colors = self
            .connection
            .prepare(&query)?
            .into_iter()
            .map(|r| r.unwrap()).collect::<Vec<_>>();
         let colors = colors.iter().map(|r| r.read::<&str, _>("color"))
            .map(|r| String::from_str(r).unwrap())
            .map(|s| Color { color: s })
            .collect::<Vec<_>>();
	Ok(colors)
    }

    fn get_database_column<'a, T>(&'a self, row: &'a [Row], column: &str) -> anyhow::Result<T>
    where
        T: TryFrom<&'a sqlite::Value, Error = sqlite::Error>,
        T: Clone,
	T: Copy
    {
        let binding  = row
            .iter()
            .map(|r| r.read::<T, _>(column))
            .collect::<Vec<_>>();
	let x = binding.first().ok_or(std::fmt::Error)?;
	Ok(*x)
    }
}
