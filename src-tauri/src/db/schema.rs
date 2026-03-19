use rusqlite::{Connection};
use crate::core::error::{AppResult};

refinery::embed_migrations!("migrations");

pub fn initialize_schema(conn: &mut Connection) -> AppResult<()> {
    migrations::runner().run(conn)?;
    Ok(())
}