use dotenvy::dotenv;
use std::{env, error, fmt};
use rand::prelude::*;
use rand::{Rng, thread_rng};
use rand::distributions::{Alphanumeric, DistString};

use rocket::{Rocket, get, routes, launch};
use rocket::tokio::time::{sleep, Duration};
use rocket::response::{status, content, Responder};
use rocket_session_store::{memory::MemoryStore, SessionStore,
	SessionResult, Session, CookieConfig};

use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_postgres::{NoTls, Error};

use comrak::{markdown_to_html, ComrakOptions};
use bbscope::BBCode;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
	let _path_to_dot_env = dotenv().ok();

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
async fn index(session: Session<'_, String>)
		-> SessionResult<content::RawHtml<String>>
{
	let session_id: String = session.get().await
		.unwrap_or_default()
		.and_then(|name| if name.len() == 54
			&& name.starts_with("jK")
			&& name.bytes().all(|x| x.is_ascii_alphanumeric())
			{Some(name)} else {None})
		.unwrap_or("jK".to_string()
		+ &Alphanumeric.sample_string(&mut rand::thread_rng(), 52));
	session.set(session_id.clone()).await?;

	let database_url = std::env::var("DATABASE_URL")
		.unwrap_or("localhost".to_string());

	let mut conn_error_string = "".to_string();
	let Ok((client, connection))
		= tokio_postgres::connect(&database_url, NoTls).await
		.or_else(|e: tokio_postgres::error::Error|
			{conn_error_string += &e.to_string(); Err(e)})
	else {return Ok(content::RawHtml(format!(
		"database connection failed: {}", conn_error_string )))};

	tokio::spawn(async move {
		if let Err(e) = connection.await {
			eprintln!("connection error: {}", e);
		}});

	let Ok(rows) = client
        .query("SELECT $1::TEXT, NOW()::TEXT", &[&"hello world"]).await
	else {return Ok(content::RawHtml("database query failed".to_string()))};

	// panics if rows[].get() aren't the right type
    Ok(content::RawHtml(format!("{}<br>{} {}", session_id,
		rows[0].get::<usize, String>(0), rows[0].get::<usize, String>(1))))
}

