#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

use rocket::{State, fairing::AdHoc, request::Form, response::Redirect};
use rusqlite::{Connection, params};

const DB_SETUP: &'static str = "CREATE TABLE IF NOT EXISTS newsletter ( id INTEGER PRIMARY KEY, email TEXT UNIQUE NOT NULL )";
const DB_INSERT: &'static str = "INSERT INTO newsletter (email) VALUES (?1)";

struct Database(String);

#[derive(FromForm)]
struct NewsletterData {
    email: String,
    #[form(field = "action")]
    _action: String,
}

#[post("/newsletter", data="<form>")]
fn newsletter(form: Form<NewsletterData>, database: State<Database>) -> Result<Redirect, rusqlite::Error> {
    if form.email == "" {
        return Ok(Redirect::to("/index.html"));
    }
    let conn = Connection::open(&database.0)?;
    conn.execute(DB_SETUP, params![])?;

    if let Err(err) = conn.execute(DB_INSERT, params![form.email]) {
        if let rusqlite::Error::SqliteFailure(sqlite_err, _) = err {
            if sqlite_err.code == rusqlite::ErrorCode::ConstraintViolation {
                Ok(Redirect::to("/index.html"))
            } else {
                Err(err)
            }
        } else {
            Err(err)
        }
    } else {
        Ok(Redirect::to("/index.html"))
    }
}

fn main() {
    rocket::ignite().mount("/", routes![newsletter]).attach(
        AdHoc::on_attach("Database Config", |rocket| {
            let database = rocket.config().get_str("database").unwrap_or("newsletter.db").to_string();

            Ok(rocket.manage(Database(database)))
        })
    ).launch();
}
