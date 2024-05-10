use sqlite::Connection;
use crate::theme_calculation::ColorTheme;

pub fn connect_to_database(path: &str) -> anyhow::Result<Connection> {
   let conn = sqlite::open(path)?;
    let query ="CREATE TABLE IF NOT EXISTS wallpaper_record(path TEXT, json TEXT);";
    conn.execute(query)?;
    Ok(conn)
}

pub fn insert_color_theme_record(conn: Connection, image_path: &str, theme: ColorTheme) -> anyhow::Result<()> {
    let theme_json = serde_json::to_string(&theme)?;
    let query = format!("INSERT INTO wallpaper_record VALUES({},{})", image_path, theme_json);
    conn.execute(query)?;
    Ok(())
}

pub fn select_color_theme_by_image_path(conn: Connection, image_path: &str) -> anyhow::Result<ColorTheme> {
    let query = format!("SELECT json FROM wallpaper_record WHERE path = {}", image_path);
    let row = conn.prepare(query)?.into_iter().bind((1, image_path))?.map(|r| r.unwrap()).next().ok_or(std::fmt::Error)?;
    let json = row.read::<&str,_>("json");
    let color_theme: ColorTheme = serde_json::from_str(json)?;
    Ok(color_theme)
}
