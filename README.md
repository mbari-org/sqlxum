# sqlxum

This repo is a learning playground for the core elements of a REST service
on top of a Postgres database, using [axum], [utoipa], and [sqlx].

[axum]: https://github.com/tokio-rs/axum
[utoipa]: https://github.com/juhaku/utoipa
[sqlx]: https://github.com/launchbadge/sqlx

There are of course several important aspects to consider when building a production service.
Some recommended resources:

- https://github.com/jeremychone-channel/rust-axum-course
- https://github.com/davidpdrsn/realworld-axum-sqlx
- https://github.com/janos-r/axum-template

## Running sqlxum

In short, the sqlxum service accepts queries from the client, submits them to the database,
and returns results in JSON format.
By default, an already running database (indicated with the `DATABASE_URL` env var) is assumed.
Responses are handled in a generic, ad hoc way, with only a very small subset of Postgres data
types is covered.

A proper database can also be set up to exercise the migration facilities of sqlx,
as well as the modeling and API facilities of axum.

Development and testing commands are captured in a [justfile] file.
(In the following, `j` is an alias for the [just] command runner.)

[justfile]: ./justfile
[just]: https://just.systems/

Running sqlxum:

- Copy `.env.template` to `.env` and adjust as needed
  (or set up the indicated env vars in some other way)
- The only required env var is `DATABASE_URL`
- Run `j run --help` to see the program usage (same as `cargo run -- --help`)
- The service per se is started with `j serve`
- Other program commands are mainly to facilitate testing without having to
  run the sqlxum server, like running SQL queries directly:
  `j query "select * from some_table limit 3"`
- Other recipes are for common development tasks, formatting, linting, etc.

## Migrations

To create and manage an own DB for testing purposes
(also designated via the `DATABASE_URL` env var),
we can use the [`sqlx-cli`] tool to initialize the DB according to the
SQL files in `migrations/`:

[`sqlx-cli`]: https://crates.io/crates/sqlx-cli

```sh
j db-setup
```

Less important, but there are some recipes to directly (not via the sqlxum service)
enter some test data into the database:
    
```sh
j db-insert-rows
```

We can now launch the sqlxum service with the `--own-db` option,
which tells the service it can start by applying any migrations that may be defined later.

```sh
j serve --own-db
```

To avoid any risks, the `--own-db` only allows `sqlxum_test` as the database name.

## OpenAPI

At startup, the service will print out the API related URLs:

```
api : http://localhost:8080/api
doc : http://localhost:8080/apidoc/
spec: http://localhost:8080/api-docs/openapi.json
Server listening on 0.0.0.0:8080
```

Because associated structs and schemas are put in place when setting up the database,
endpoints like `/api/users` are only meaningful when using the `--own-db` option.

One can now use the UI at `http://localhost:8080/apidoc/` to interact with the database.

Some examples via the command line using curlie:

```sh
curlie get http://localhost:8080/api/users
curlie get http://localhost:8080/api/users where=='name = `Foo`'
curlie get http://localhost:8080/api/users limit==1
curlie get http://localhost:8080/api/users where=='created_at > `20240218T03:40`'
curlie post http://localhost:8080/api/users email=foo@example.net name='Foo Bar'
curlie delete http://localhost:8080/api/users userId=c46c29d6-ce29-11ee-af0c-73c279b2e1ce
```

> The backticks are a convenience to facilitate escaping in the shell.
> They are replaced for single quotes before the actual submission to the database.
