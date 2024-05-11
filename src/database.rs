#![deny(unused_extern_crates)]
#![warn(missing_docs)]
use crate::theme_calculation::ColorTheme;
use sqlite::Connection;
use std::path::{PathBuf, Path};

pub struct DatabaseConnection {
    connection: Connection,
}

impl DatabaseConnection {
    pub fn new(path: &PathBuf) -> anyhow::Result<DatabaseConnection> {
        let conn = sqlite::open(path)?;
        let query = "CREATE TABLE IF NOT EXISTS wallpaper_record(path TEXT, json TEXT);";
        conn.execute(query)?;
        Ok(DatabaseConnection { connection: conn })
    }

    pub fn insert_color_theme_record(
        &self,
        image_path: &Path,
        theme: &[ColorTheme],
    ) -> anyhow::Result<()> {
        let theme_json = serde_json::to_string(&theme)?;
        let query = format!(
            "INSERT INTO wallpaper_record (path, json) VALUES ('{}','{}')",
            image_path.to_str().ok_or(std::fmt::Error)?,
            theme_json
        );
        self.connection.execute(query)?;
        Ok(())
    }

    pub fn select_color_theme_by_image_path(
        &self,
        image_path: &Path,
    ) -> anyhow::Result<Vec<ColorTheme>> {
        let query = format!(
            "SELECT json FROM wallpaper_record WHERE path = '{}'",
            image_path.to_str().ok_or(std::fmt::Error)?
        );
        let row = self
            .connection
            .prepare(&query)?
            .into_iter()
            .map(|r| r.unwrap())
            .collect::<Vec<_>>();
        let json = row
            .iter()
            .map(|r| r.read::<&str, _>("json"))
            .collect::<String>();
        let color_theme: Vec<ColorTheme> = serde_json::from_str(&json)?;
        Ok(color_theme)
    }
}
