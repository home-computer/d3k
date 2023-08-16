# > *delurker_3000*

Sample .env:

```ini
# production

TELOXIDE_TOKEN=<token goes here>
DATABASE_URL=postgres://d3k:password@0:5432/d3k

# development

# used by the migration tool
FLYWAY_URL=jdbc:postgresql://localhost:5432/d3k?user=d3k&password=password

RUST_LOG=info
RUST_LOG_TEST=info,sqlx=warn

# used to configure the dev postgres servers
DB_USERNAME=d3k
DB_PASSWORD=password

# used by the unit tests
TEST_DB_USER=d3k
TEST_DB_PASS=password
TEST_DB_HOST=localhost
TEST_DB_PORT=5432
```
