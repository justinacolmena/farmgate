use dotenvy::dotenv;
use rand::distributions::{Alphanumeric, DistString};
use std::time::SystemTime;
use std::convert::Infallible;
use chrono::{DateTime, offset};
use rocket::{get, routes, Request};
use rocket::http::{Header, Status, ContentType, HeaderMap};
use rocket::request::{FromRequest, Outcome};
use rocket::response::Responder;
use std::collections::HashMap;
use secrecy::{Secret, SecretString, ExposeSecret};
use rocket_session_store::{memory::MemoryStore, SessionStore,
	SessionResult, Session, CookieConfig};
use rocket_db_pools::{deadpool_postgres, Database, Connection};

#[derive(Database)]
#[database("PostgreSQL")]
struct Db(deadpool_postgres::Pool);

#[cfg(feature = "derive")]
use postgres_types::{ToSql, FromSql};
// use comrak::{markdown_to_html, ComrakOptions};
// use bbscope::BBCode;


#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
	let _path_to_dot_env = dotenv().ok();

	let memory_store: MemoryStore::<HashMap<String,SecretString>> = MemoryStore::default();
	let store: SessionStore<HashMap<String,SecretString>> = SessionStore {
		store: Box::new(memory_store),
		name: "token".into(),
		duration: tokio::time::Duration::from_secs(3600),
		cookie: CookieConfig::default(),
	};
	let _rocket = rocket::build()
		.attach(store.fairing())
		.attach(Db::init())
		.mount("/", routes![index,login_auth])
		.launch().await?;
	Ok(())
}


async fn session_init(session: Session<'_, HashMap<String,SecretString>>, key: &str, len: usize)
		-> SessionResult<HashMap<String,SecretString>> {
	let mut sh: HashMap<String,SecretString> = session.get().await?
		.or_else(||Some(HashMap::new())).unwrap_or_else(||HashMap::new());
	if sh.get(key).and_then(|val|Some(val.expose_secret()))
		.and_then(|ss|Some(ss.len()==len && ss.bytes().all(|x|x.is_ascii_alphanumeric())))
		.unwrap_or(false) { session.touch().await? }
	else {
		sh.insert(key.to_string(),
			Secret::new(Alphanumeric.sample_string(&mut rand::thread_rng(), len)));
		session.set(sh.clone()).await?
	}
	Ok(sh)
}


#[get("/")]
async fn index(session: Session<'_, HashMap<String,SecretString>>,
			db: Connection<Db>, http_request_headers: HttpRequestHeaders<'_>)
		-> SessionResult<(Status, (ContentType, String))>
{
	let mut session_vars = session_init(session, "secret_key", 54).await?;
	// Don't forget "session.set(session_vars).await?" to finalize any modifications!
	let secret_key: SecretString = if let Some(key) = session_vars.get("secret_key")
	{(*key).clone()} else {Secret::new(String::new())};
	// Either a cloned copy of the secret key string or a new secret empty string.

	let mut database_error = String::new();

	let Ok(rows) = db.query("SELECT $3, $2, $1, NOW()",
			&[&"world", &"hello", &secret_key.expose_secret()]).await
	.or_else(|e: tokio_postgres::error::Error|
			{database_error += &e.to_string(); Err(e)})
	else {return Ok((Status::new(500), (ContentType::Plain,
		format!("database query failed: {}\n", database_error))))};

	// use non-panic method & trap errors with "?" operator inside closure
    (move |rows:Vec<tokio_postgres::row::Row>| async move {
		let mut r = String::new();
		for row in rows {
			r += &format!("{}<br>\n{} {} {}<br>\n<a href=\"/login/auth\">login</a>: \
			try username “aladdin” with password “opensesame”",
				row.try_get::<_,&str>(0)?,
				row.try_get::<_,&str>(1)?,
				row.try_get::<_,&str>(2)?,
				DateTime::<offset::Utc>::from(row.try_get::<_,SystemTime>(3)?)
			)}
		Ok((Status::Ok,(ContentType::HTML, r)))
	})(rows).await // call the closure on "rows" returned from database
	.or_else(|e: tokio_postgres::error::Error|
		Ok((Status::new(500), (ContentType::Plain,
		format!("failed to get results from database query: {}\n",
			&e.to_string())))))
}

// https://github.com/SergioBenitez/Rocket/discussions/2041#discussion-3770847
struct HttpRequestHeaders<'h>(&'h HeaderMap<'h>);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for HttpRequestHeaders<'r> {
    type Error = Infallible;
    async fn from_request(http_request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let http_request_headers = http_request.headers();
        Outcome::Success(HttpRequestHeaders(http_request_headers))
    }
}

// https://www.reddit.com/r/rust/comments/oy37e5/comment/h7s7w62/
// https://rocket.rs/v0.5-rc/guide/responses/#custom-responders
#[derive(Responder)]
struct MyResponder<T> {
    inner: T,
    my_header: Header<'static>,
}

impl<'r, 'o: 'r, T: Responder<'r, 'o>> MyResponder<T> {
    fn new(inner: T, header_key: &'static str, header_value: &'static str) -> Self {
        MyResponder {
            inner,
            my_header: Header::new(header_key, header_value),
        }
    }
}

// The "/login/auth" page has no content of its own. This preliminary version implements
// RFC 7617 Basic Authentication for login with username "aladdin" and password
// "opensesame". If successful, the user is redirected to the home page "/".
// Basic Authentication strips off the portion of the URI after the final '/' to
// obtain the root of the protection domain for reusing credentials.
//
// The plan is to implement RFC 7616 Digest Access Authentication with the "domain"
// parameter restricted to "/login/" and rely on rocket-session-store and the database
// to manage sessions and carry user authentication over to the rest of the website.

#[get("/login/auth")]
async fn login_auth(session: Session<'_, HashMap<String,SecretString>>,
			db: Connection<Db>, http_request_headers: HttpRequestHeaders<'_>)
		-> SessionResult<MyResponder<(Status, ())> > {

	let auth = http_request_headers.0.get_one("Authorization").unwrap_or("");
    let auth64 = if auth.len() > 6 && auth.starts_with("Basic") { auth[5..].trim() } else {""};
    let user_pass_vu8 = base64::decode(&auth64).unwrap_or_default();
	let user_pass_str =  std::str::from_utf8(&user_pass_vu8).expect("username:password");

	if user_pass_str == "aladdin:opensesame" {

		Ok(MyResponder::new((Status::TemporaryRedirect, ()), "Location", "/"))
	}
		else {
		Ok(MyResponder::new((Status::Unauthorized, ()),
								 "WWW-Authenticate", "Basic realm=\"login\", charset=\"UTF-8\""))
	}
}
