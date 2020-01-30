use diesel::prelude::*;
use diesel::r2d2;
use diesel::r2d2::ConnectionManager;
use failure::Error;
use std::path::PathBuf;
use std::{fs, fs::File};

type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

lazy_static! {
    static ref DB_PATH: PathBuf = glib::get_user_data_dir().unwrap().join("read-it-later");
    static ref POOL: Pool = init_pool().expect("Failed to create a pool");
}

embed_migrations!("migrations/");

pub(crate) fn connection() -> Pool {
    POOL.clone()
}

fn run_migration_on(connection: &SqliteConnection) -> Result<(), Error> {
    info!("Running DB Migrations...");
    embedded_migrations::run_with_output(connection, &mut std::io::stdout()).map_err(From::from)
}

fn init_pool() -> Result<Pool, Error> {
    let db_path = &DB_PATH;
    fs::create_dir_all(&db_path.to_str().unwrap())?;
    let db_path = db_path.join("articles.db");
    if !db_path.exists() {
        File::create(&db_path.to_str().unwrap())?;
    }
    let manager = ConnectionManager::<SqliteConnection>::new(db_path.to_str().unwrap());
    let pool = r2d2::Pool::builder().max_size(1).build(manager)?;

    {
        let db = pool.get()?;
        run_migration_on(&*db)?;
    }
    info!("Database pool initialized.");
    Ok(pool)
}

pub trait Insert<T> {
    type Error;

    fn insert(&self) -> Result<T, Self::Error>;
}
