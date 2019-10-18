


pub fn get_articles() -> Result<Vec<Article>, Error> {
    use crate::schema::chapters::dsl::*;
    let db = connection();
    let conn = db.get()?;

    chapters.load::<Chapter>(&conn).map_err(From::from)
}


