# farmgate
A CMS with user auth, blog, wiki, and discussion to run on PostgreSQL

Configure `DATABASE_URL` and `ROCKET_SECRET_KEY` in the file `./.env`

**.env**

```
DATABASE_URL = postgres://username:password@localhost/somedatabase
ROCKET_SECRET_KEY = qjsVtkPNv7903JcCwbSxNl68cMT9F8D6h834RoDNSSI=
```
The variable `ROCKET_SECRET_KEY` may be set to the output of the
following command to generate a new secret key

```
$ head -c32 /dev/random | base64
```
