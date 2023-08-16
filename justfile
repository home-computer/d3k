set shell := ["sh", "-c"]
set dotenv-load

# List the avail commands
default:
  @just --list --unsorted

pre-commit:
  cargo fmt
  @just gen-sqlx-offline

CARGO_TARGET_DIR := "/var/run/media/asdf/Windows/target/delurker_3000/" 
SQLX_TMP := "/var/run/media/asdf/Windows/target/delurker_3000/sqlx-tmp" 
SQLX_OFFLINE_DIR := "/var/run/media/asdf/Windows/target/delurker_3000/sqlx-final"
gen-sqlx-offline:
  mkdir -p .sqlx && mkdir -p {{SQLX_TMP}} && mkdir -p {{SQLX_OFFLINE_DIR}}
  rm .sqlx/query-*.json  || true
  # force full recomplile of crates that use sqlx queries
  cargo clean -p delurker_3000
  SQLX_TMP={{SQLX_TMP}} SQLX_OFFLINE_DIR={{SQLX_OFFLINE_DIR}} cargo check
  cp {{SQLX_OFFLINE_DIR}}/* .sqlx -r

r:
  cargo run

# psql from the db running in the compose launched pg container
psql *ARGS:
  podman-compose -f ./docker-compose.yml -f ./docker-compose.dev.yml \
    exec postgres \
    psql {{ARGS}}

# psql command but fit for redirects
psql-tty *ARGS:
  podman-compose -f ./docker-compose.yml -f ./docker-compose.dev.yml \
    exec -T postgres \
    psql {{ARGS}}

# The flyway cli tool
flyway *ARGS:
  podman-compose --profile tools run --rm \
    flyway {{ARGS}}

# Apply migrations to database.
db-mig:
  cargo sqlx database create
  @just flyway migrate

# Seed db from the fixtures.
db-seed:
  @just psql-tty < fixtures/000_test_data.sql  

# Apply migrations to database.
db-reset:
  cargo sqlx database drop -y
  @just db-mig 
  @just db-seed

alias dev := dev-up

# Start all services required for development
dev-up:
  podman-compose -f docker-compose.yml -f docker-compose.dev.yml up -d

dev-down *ARGS:
  podman-compose -f docker-compose.yml -f docker-compose.dev.yml down {{ARGS}}

logs-dev:
  podman-compose -f docker-compose.yml -f docker-compose.dev.yml logs -f -n -t

test *ARGS:
  cargo nextest run {{ARGS}}
