set dotenv-load := true

# List recipes
list:
  @just --list --unsorted

###################################################################################
## DB setup and direct queries

# Set up database 'sqlxum_test' using sqlx-cli
db-setup:
    sqlx db setup --database-url "postgresql://postgres:123@localhost/sqlxum_test"

# Insert some data in the test database
db-insert-rows:
    just db-insert-user 'foo@example.net' 'Foo'

# Insert a user in the test database
db-insert-user email name:
    just query 'insert into usr (email, name) values(`{{email}}`, `{{name}}`)'

###################################################################################
## Commands using the API

# GET /api/users
get-users *args='':
    curlie get http://localhost:8080/api/users '{{args}}'

# POST /api/users
post-users *args='':
    curlie post http://localhost:8080/api/users {{args}}

# PUT /api/users
put-users *args='':
    curlie put http://localhost:8080/api/users {{args}}

# DELETE /api/users
delete-user *args='':
    curlie -v delete http://localhost:8080/api/users {{args}}


###################################################################################
## Program commands

# Launch server
serve *args='':
    cargo run -- serve {{args}}

# Launch server with --own-db
serve-own-db *args='':
    cargo run -- serve --own-db {{args}}

# Direct query to database
query query:
    cargo run -- db --query '{{query}}'

# Direct health check
health:
    cargo run -- health


###################################################################################
## Some convenient dev recipes in general

# Good to run before committing changes
all: test format clippy

# cargo check
check:
    cargo check

# cargo watch
watch *cmd='check':
    cargo watch -c -x '{{ cmd }}'

# Run tests
test *args='':
    cargo test {{args}}

# Run tests (using only one thread)
test-1-thread *args='':
    cargo test --jobs=1 -- --test-threads=1 {{args}}

# Run tests, --nocapture
test-nocapture *args='':
    just test --nocapture {{args}}

# cargo run
run *args='':
    cargo run -- {{args}}

# rm -rf target/
clean:
    rm -rf target

# Format source code
format:
    cargo fmt

# cargo clippy
clippy:
    cargo clippy --all-targets -- -D warnings

# cargo build
build *args='--release':
    cargo build {{args}}

# (cargo install cargo-modules)
# Show module structure
structure package='sqlxum':
    cargo modules structure --package {{package}}

# Show module dependencies
dependencies package='sqlxum':
    cargo modules dependencies --package {{package}}

# (cargo install --locked cargo-outdated)
# Show outdated dependencies
outdated:
    cargo outdated --root-deps-only

# (cargo install --locked cargo-udeps)
# Find unused dependencies
udeps:
    cargo +nightly udeps

# cargo update
update:
    cargo update
