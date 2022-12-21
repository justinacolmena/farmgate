use rocket::{Rocket, get, routes, launch};
use rocket::tokio::time::{sleep, Duration};
use rocket::response::content;

use rand::prelude::*;
use rand::{Rng, thread_rng};
use rand::distributions::{Alphanumeric, DistString};
use rocket_session_store::{memory::MemoryStore, SessionStore,
	SessionResult, Session, CookieConfig};

use dotenvy::dotenv;
use std::env;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_postgres::{NoTls, Error};

use comrak::{markdown_to_html, ComrakOptions};
use bbscope::BBCode;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {

    let memory_store: MemoryStore::<String> = MemoryStore::default();
	let store: SessionStore<String> = SessionStore {
		store: Box::new(memory_store),
		name: "token".into(),
		duration: Duration::from_secs(3600),
		// The cookie config is used to set the cookie's path and other options.
		cookie: CookieConfig::default(),
	};
   let _rocket = rocket::build().attach(store.fairing())
        .mount("/", routes![index])
        .launch()
        .await?;

    Ok(())
}

#[get("/")]
async fn index(session: Session<'_, String>) -> SessionResult<content::RawHtml<String>> {

	let name: Option<String> = session.get().await?;
	if let Some(name) = name {
		session.set(name).await?;
	} else {
		let name = Alphanumeric.sample_string(&mut rand::thread_rng(), 52);
		session.set(name).await?;
	}

	let database_url = dotenvy::var("DATABASE_URL")
		.or(std::env::var("DATABASE_URL"))
		.or(Ok::<String,Error>("localhost".to_string())).unwrap();

	let Ok((client, connection)) = tokio_postgres::connect(&database_url, NoTls).await
	else {return Ok(content::RawHtml("database connection failed".to_string()))};
	tokio::spawn(async move {
		if let Err(e) = connection.await {
			eprintln!("connection error: {}", e);
		}
	});
	let Ok(rows) = client
        .query("SELECT $1::TEXT, NOW()::TEXT", &[&"hello world"])
        .await
	else {return Ok(content::RawHtml("database query failed".to_string()))};

	// panics if rows[].get() aren't the right type
    Ok(content::RawHtml(format!("{}<br>{} {}", session.get().await.unwrap().unwrap(),
		rows[0].get::<usize, String>(0), rows[0].get::<usize, String>(1))))
}

