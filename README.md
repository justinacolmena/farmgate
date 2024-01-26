# farmgate
A CMS with user auth, blog, wiki, and discussion to run on PostgreSQL

Configure `ROCKET_SECRET_KEY` and `ROCKET_DATABASES` in the file `./.env`

**.env**

```
ROCKET_SECRET_KEY = qjsVtkPNv7903JcCwbSxNl68cMT9F8D6h834RoDNSSI=
ROCKET_DATABASES = '{PostgreSQL={url="postgres://user:password@localhost/database",idle_timeout=120}}'
```
The variable `ROCKET_SECRET_KEY` may be set to the output of the
following command to generate a new secret key for encrypting session cookies

```
$ head -c32 /dev/random | base64
```
# TO DO:
 * session support (done!)
 * login page with 401 Digest Auth and 307 redirect to referer if local (works with Basic auth!)
 * simple tables in database for Sessions, Users, & Content
 * pages for signup, logout, user preferences, profile
 * actions for view, edit, post, update, history, delete
 * forms, views, html, css, styles, skins, navigation help
 * Bring diesel onboard: https://crates.io/crates/diesel
