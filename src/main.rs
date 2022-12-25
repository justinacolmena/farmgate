use dotenvy::dotenv;
use rand::distributions::{Alphanumeric, DistString};
use rocket::{get, routes};
use rocket::tokio::time::{Duration};
use rocket::http::{Status, ContentType};
use rocket_session_store::{memory::MemoryStore, SessionStore,
	SessionResult, Session, CookieConfig};
use tokio_postgres::{NoTls};

// use comrak::{markdown_to_html, ComrakOptions};
// use bbscope::BBCode;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
	let _path_to_dot_env = dotenv().ok();

	let memory_store: MemoryStore::<String> = MemoryStore::default();
	let store: SessionStore<String> = SessionStore {
		store: Box::new(memory_store),
		name: "token".into(),
		duration: Duration::from_secs(3600),
		cookie: CookieConfig::default(),
	};
	let _rocket = rocket::build().attach(store.fairing())
		.mount("/", routes![index])
		.launch()
		.await?;
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
		.unwrap_or("postgres://localhost".to_string());

	let mut database_error = "".to_string();
	let Ok((client, connection))
		= tokio_postgres::connect(&database_url, NoTls).await
		.or_else(|e: tokio_postgres::error::Error|
			{database_error += &e.to_string(); Err(e)})
	else {return Ok((Status::new(500), (ContentType::Plain,
		format!("database connection failed: {}", database_error))))};

	tokio::spawn(async move {
		if let Err(e) = connection.await {
			eprintln!("connection error: {}", e); }});

	let Ok(rows) = client
        .query("SELECT $1::TEXT, $2::TEXT, NOW()::TEXT",
			&[&&session_id, &"hello world"]).await
	.or_else(|e: tokio_postgres::error::Error|
			{database_error += &e.to_string(); Err(e)})
	else {return Ok((Status::new(500), (ContentType::Plain,
		format!("database connection failed: {}", database_error))))};

	// panics if rows[].get() aren't the right type
    Ok((Status::Ok,(ContentType::HTML, format!("{}<br>{} {}",
		rows[0].get::<usize, String>(0), rows[0].get::<usize, String>(1),
			rows[0].get::<usize, String>(2)))))
}

