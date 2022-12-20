use rocket::{
	Rocket,
	get,
	routes,
	launch,
};

use rocket_session_store::{
	memory::MemoryStore,
	SessionStore,
	SessionResult,
	Session,
	CookieConfig,
};


use dotenvy::dotenv;
use std::env;
use rand::prelude::*;


use rocket::tokio::time::{sleep, Duration};
use rocket::response::content;
use comrak::{markdown_to_html, ComrakOptions};
use bbscope::BBCode;



#[rocket::main]
async fn main() -> Result<(), rocket::Error> {

    let memory_store: MemoryStore::<String> = MemoryStore::default();
	let store: SessionStore<String> = SessionStore {
		store: Box::new(memory_store),
		name: "token".into(),
		duration: Duration::from_secs(3600 * 24 * 3),
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
async fn index(session: Session<'_, String>) -> SessionResult<String> {
	let name: Option<String> = session.get().await?;
	if let Some(name) = name {
		Ok(format!("Hello, {}!", name))
	} else {
		session.set(format!("justina")).await?;
		Ok("Hello, world!".into())
	}
}

