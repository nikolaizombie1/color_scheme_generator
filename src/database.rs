#![deny(unused_extern_crates)]
#![warn(missing_docs)]
use crate::theme_calculation::ColorTheme;
use sqlite::Connection;
use std::path::{PathBuf, Path};

/// Hold a sqlite database connection.
pub struct DatabaseConnection {
    connection: Connection,
}

impl DatabaseConnection {
    /// Create new sqlite database connection, creating database if it does not exist,
    /// and create table wallpaper_record table if it does not exist.
    ///
    /// Returns an error if the database cannot be created or written to.
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use color_scheme_generator::database::DatabaseConnection;
    /// let db_path = "test.db".parse::<PathBuf>().unwrap();
    /// let conn = DatabaseConnection::new(&db_path).unwrap();
    /// ```
    pub fn new(path: &PathBuf) -> anyhow::Result<DatabaseConnection> {
        let conn = sqlite::open(path)?;
        let query = "CREATE TABLE IF NOT EXISTS wallpaper_record(path TEXT, json TEXT);";
        conn.execute(query)?;
        Ok(DatabaseConnection { connection: conn })
    }

    /// Insert a list of [`ColorTheme`] into the database.
    ///
    /// Should never panic or error but could if connection
    /// to the sqlite database is lost or if the image path could
    /// not be converted to a String.
    /// # Examples
    ///
    /// ```
    /// # use std::path::PathBuf;
    /// # use color_scheme_generator::database::DatabaseConnection;
    /// # use color_scheme_generator::theme_calculation::{ColorTheme, RGB};
    /// # let db_path = "test.db".parse::<PathBuf>().unwrap();
    /// let conn = DatabaseConnection::new(&db_path).unwrap();
    /// # let themes = vec![ColorTheme{bar_color: RGB{red: 1, green: 2, blue: 3}, workspace_color:RGB{red: 4, green: 5, blue: 6}, text_color: RGB{red: 7, green: 8, blue: 9}}];
    /// # let image_path = "PATH".parse::<PathBuf>().unwrap();
    /// conn.insert_color_theme_record(&image_path, &themes).unwrap();
    /// ```
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

    /// Select color theme from database given an image path.
    ///
    /// Will return an error if the image path is not found in the database
    /// or if the image path cannot be converted to a string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::path::PathBuf;
    /// # use color_scheme_generator::database::DatabaseConnection;
    /// # use color_scheme_generator::theme_calculation::{ColorTheme, RGB};
    /// # let db_path = "test.db".parse::<PathBuf>().unwrap();
    /// let conn = DatabaseConnection::new(&db_path).unwrap();
    /// # let image_path = "PATH".parse::<PathBuf>().unwrap();
    /// conn.select_color_theme_by_image_path(&image_path);
    /// ```
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
