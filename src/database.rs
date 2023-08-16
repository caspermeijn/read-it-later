use std::{fs, fs::File, path::PathBuf};

use anyhow::Result;
use diesel::{prelude::*, r2d2, r2d2::ConnectionManager};
use glib::once_cell::sync::Lazy;
use gtk::glib;
use log::info;

type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

static DB_PATH: Lazy<PathBuf> = Lazy::new(|| glib::user_data_dir().join("read-it-later"));
static POOL: Lazy<Pool> = Lazy::new(|| init_pool().expect("Failed to create a Pool"));

embed_migrations!("migrations/");

pub(crate) fn connection() -> Pool {
    POOL.clone()
}

fn run_migration_on(connection: &SqliteConnection) -> Result<()> {
    info!("Running DB Migrations...");
    embedded_migrations::run_with_output(connection, &mut std::io::stdout()).map_err(From::from)
}

fn init_pool() -> Result<Pool> {
    fs::create_dir_all(&*DB_PATH)?;
    let db_path = DB_PATH.join("articles.db");

    if !db_path.exists() {
        File::create(&db_path)?;
    }
    let manager = ConnectionManager::<SqliteConnection>::new(db_path.to_str().unwrap());
    let pool = r2d2::Pool::builder().max_size(1).build(manager)?;

    {
        let db = pool.get()?;
        run_migration_on(&db)?;
    }
    info!("Database pool initialized.");
    Ok(pool)
}

pub fn wipe() -> Result<()> {
    let db = connection();
    let conn = db.get()?;
    use crate::schema::articles::dsl::*;

    diesel::delete(articles).execute(&conn)?;
    Ok(())
}

pub trait Insert<T> {
    type Error;

    fn insert(&self) -> Result<T, Self::Error>;
}
