use dotenvy::dotenv;
use rand::distributions::{Alphanumeric, DistString};
use std::time::SystemTime;
use chrono::DateTime;
use chrono::offset::{Utc};
use rocket::{get, routes};
use rocket::http::{Status, ContentType};
use rocket_session_store::{memory::MemoryStore, SessionStore,
	SessionResult, Session, CookieConfig};
use tokio_postgres::{NoTls};

#[cfg(feature = "derive")]
use postgres_types::{ToSql, FromSql};

// use comrak::{markdown_to_html, ComrakOptions};
// use bbscope::BBCode;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
	let _path_to_dot_env = dotenv().ok();

	let memory_store: MemoryStore::<String> = MemoryStore::default();
	let store: SessionStore<String> = SessionStore {
		store: Box::new(memory_store),
		name: "token".into(),
		duration: tokio::time::Duration::from_secs(3600),
		cookie: CookieConfig::default(),
	};
	let _rocket = rocket::build().attach(store.fairing())
		.mount("/", routes![index]).launch().await?;
	Ok(())
}

async fn session_init(session: Session<'_, String>) -> SessionResult<String>
{
	let session_id: String = session.get().await.unwrap_or_default()
		.and_then(|name| if name.len() == 54 && name.bytes()
		.all(|x| x.is_ascii_alphanumeric()) {Some(name)} else {None})
		.unwrap_or_else(||Alphanumeric.sample_string(&mut rand::thread_rng(), 54));
	session.set(session_id.clone()).await.and_then(|()|Ok(session_id))
}

#[get("/")]
async fn index(session: Session<'_, String>)
			-> SessionResult<(Status, (ContentType, String))> {
	let session_id: String = session_init(session).await?;

	let database_url = std::env::var("DATABASE_URL")
		.unwrap_or("postgresql://localhost".to_string());

	let mut database_error = "".to_string();
	let Ok((client, connection))
		= tokio_postgres::connect(&database_url, NoTls).await
		.or_else(|e: tokio_postgres::error::Error|
			{database_error += &e.to_string(); Err(e)})
	else {return Ok((Status::new(500), (ContentType::Plain,
		format!("database connection failed: {}", database_error))))};

	tokio::spawn(async move {
		if let Err(e) = connection.await {
			eprintln!("database connection error: {}", e); }});

	let Ok(rows) = client
        .query("SELECT $1, $2, $3, NOW()",
			&[&&session_id, &"hello", &"world"]).await
	.or_else(|e: tokio_postgres::error::Error|
			{database_error += &e.to_string(); Err(e)})
	else {return Ok((Status::new(500), (ContentType::Plain,
		format!("database query failed: {}", database_error))))};

	// use non-panic method & trap errors with "?" operator inside closure
    (|rows:Vec<tokio_postgres::row::Row>| {
		let mut r : String = "".to_string();
		for row in rows {
			r += &format!("{}\n{} {} {}\n",
				row.try_get::<usize,String>(0)?,
				row.try_get::<usize,String>(1)?,
				row.try_get::<usize,String>(2)?,
				DateTime::<Utc>::from(row.try_get::<usize,SystemTime>(3)?)
			)}
		Ok((Status::Ok,(ContentType::Plain, r)))
	})(rows) // call the closure on "rows" returned from database
	.or_else(|e: tokio_postgres::error::Error|
		Ok((Status::new(500), (ContentType::Plain,
		format!("failed to get results from database query: {}",
			&e.to_string())))))
}
