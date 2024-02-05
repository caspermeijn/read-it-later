// Copyright 2019 Bilal Elmoussaoui <bil.elmoussaoui@gmail.com>
// Copyright 2020 Julian Hofer <julian.git@mailbox.org>
// Copyright 2021 Alistair Francis <alistair@alistair23.me>
// Copyright 2022 Casper Meijn <casper@meijn.net>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{fs, fs::File, path::PathBuf};

use anyhow::{Ok, Result};
use diesel::{migration::MigrationVersion, prelude::*, r2d2, r2d2::ConnectionManager};
use diesel_migrations::EmbeddedMigrations;
use glib::once_cell::sync::Lazy;
use gtk::glib;
use log::info;

use crate::diesel_migrations::MigrationHarness;

type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

static DB_PATH: Lazy<PathBuf> = Lazy::new(|| glib::user_data_dir().join("read-it-later"));
static POOL: Lazy<Pool> = Lazy::new(|| init_pool().expect("Failed to create a Pool"));

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

pub(crate) fn connection() -> Pool {
    POOL.clone()
}

fn run_migration_on(connection: &mut SqliteConnection) -> Result<Vec<MigrationVersion<'_>>> {
    info!("Running DB Migrations...");

    connection
        .run_pending_migrations(MIGRATIONS)
        .map_err(|e| anyhow::anyhow!(e))
}

fn run_preview_migration_on(connection: &mut SqliteConnection) -> Result<()> {
    use crate::{models::Article, schema::articles::dsl::*};
    dbg!("Starting preview migration...");

    let to_be_processed: Vec<Article> = articles
        .filter(preview_text.is_null())
        .get_results::<Article>(connection)?;

    if !to_be_processed.is_empty() {
        info!("Running preview migration...");

        for article in to_be_processed {
            let new_preview_text = Article::calculate_preview(&article.content.unwrap_or_default());
            let target = articles.filter(id.eq(&article.id));
            diesel::update(target)
                .set(preview_text.eq(new_preview_text))
                .execute(connection)?;
        }
    }

    Ok(())
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
        let mut db = pool.get()?;
        run_migration_on(&mut db)?;
        run_preview_migration_on(&mut db)?;
    }
    info!("Database pool initialized.");
    Ok(pool)
}

pub fn wipe() -> Result<()> {
    let db = connection();
    let mut conn = db.get()?;
    use crate::schema::articles::dsl::*;

    diesel::delete(articles).execute(&mut conn)?;
    Ok(())
}

pub trait Insert<T> {
    type Error;

    fn insert(&self) -> Result<T, Self::Error>;
}
