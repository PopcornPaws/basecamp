use sqlx::migrate::Migrator;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use sqlx::{ConnectOptions, Error as SqlxError, Executor, PgPool};
use tracing::log::LevelFilter;

use std::time::Duration;

/// Postgres specific database configuration parameters.
#[derive(Clone, Debug)]
pub struct Config {
    pub pg_connect_opts: PgConnectOptions,
    pub pg_pool_opts: PgPoolOptions,
}

impl Default for Config {
    fn default() -> Self {
        let pg_connect_opts = PgConnectOptions::new()
            .port(5432)
            .host("localhost")
            .database("postgres")
            .username("postgres")
            .password("password")
            .ssl_mode(PgSslMode::Prefer)
            .log_statements(LevelFilter::Trace);
        let pg_pool_opts = PgPoolOptions::new()
            .acquire_timeout(Duration::from_secs(5))
            .idle_timeout(Duration::from_secs(2));

        Self {
            pg_connect_opts,
            pg_pool_opts,
        }
    }
}

impl Config {
    /// Attempts to read database config from environment variables.
    ///
    /// If an environment variable is not set, the respective config is set to its default value.
    ///
    /// # Panics
    ///
    /// Panics if parameters cannot be parsed or if the database url is provided in the wrong
    /// format (expected: `postgres://<username>:<password>@<hostname>:<port>/<database_name>`).
    #[must_use]
    pub fn from_env() -> Self {
        let mut config = Self::default();

        dotenv::dotenv().ok();
        // try to parse the whole database url if present
        if let Ok(url) = dotenv::var("DATABASE_URL") {
            config.pg_connect_opts = url.parse().expect("invalid database url");
        } else {
            if let Ok(port) = dotenv::var("DB_PORT") {
                config.pg_connect_opts = config.pg_connect_opts.port(port.parse().unwrap());
            }
            if let Ok(host) = dotenv::var("DB_HOST") {
                config.pg_connect_opts = config.pg_connect_opts.host(&host);
            }
            if let Ok(name) = dotenv::var("DB_NAME") {
                config.pg_connect_opts = config.pg_connect_opts.database(&name);
            }
            if let Ok(username) = dotenv::var("DB_USERNAME") {
                config.pg_connect_opts = config.pg_connect_opts.username(&username);
            }
            if let Ok(password) = dotenv::var("DB_PASSWORD") {
                config.pg_connect_opts = config.pg_connect_opts.password(&password);
            }
        }
        if let Ok(s) = dotenv::var("DB_REQUIRE_SSL") {
            if s.parse().unwrap() {
                config.pg_connect_opts = config.pg_connect_opts.ssl_mode(PgSslMode::Require);
            }
        }
        if let Ok(log_level) = dotenv::var("DB_LOG_LEVEL") {
            config.pg_connect_opts = config
                .pg_connect_opts
                .log_statements(log_level.parse().unwrap());
        }
        if let Ok(seconds) = dotenv::var("DB_ACQUIRE_TIMEOUT") {
            let duration = Duration::from_secs(seconds.parse().unwrap());
            config.pg_pool_opts = config.pg_pool_opts.acquire_timeout(duration);
        }
        if let Ok(seconds) = dotenv::var("DB_IDLE_TIMEOUT") {
            let duration = Duration::from_secs(seconds.parse().unwrap());
            config.pg_pool_opts = config.pg_pool_opts.idle_timeout(duration);
        }

        config
    }

    #[must_use]
    pub fn with_database(self, db: &str) -> Self {
        Self {
            pg_connect_opts: self.pg_connect_opts.database(db),
            pg_pool_opts: self.pg_pool_opts,
        }
    }

    fn connect_lazy_with(self) -> PgPool {
        self.pg_pool_opts.connect_lazy_with(self.pg_connect_opts)
    }

    fn connect_with_db(self, db: &str) -> PgPool {
        self.with_database(db).connect_lazy_with()
    }

    fn connect_without_db(self) -> PgPool {
        self.connect_with_db("")
    }

    /// Attempts to establish a connection to Postgres.
    ///
    /// # Errors
    ///
    /// Errors if the connection fails or the database cannot be created.
    pub async fn connect(self) -> Result<PgPool, SqlxError> {
        // we need to clone this because `self.connect_with...` consumes self
        // `get_database` returns a reference
        let db = self
            .pg_connect_opts
            .get_database()
            .map_or_else(|| "postgres".to_string(), ToOwned::to_owned);
        // connect to postgres without specifying custom db name
        let pool = self.clone().connect_without_db();

        // check whether the requested database exists
        let maybe_already_db = pool
            .execute(format!("SELECT 1 FROM pg_database WHERE datname='{db}'").as_str())
            .await?;

        // if database does not exist, create it
        if maybe_already_db.rows_affected() < 1 {
            tracing::info!("database \"{db}\" does not exist, creating it");

            pool.execute(format!("CREATE DATABASE \"{db}\";").as_str())
                .await?;

            tracing::info!("database created");
        } else {
            tracing::warn!("database \"{}\" already exists", db);
        }
        // connect to the db that we created
        Ok(self.connect_with_db(&db))
    }

    /// Attempts to establish a connection to Postgres and run the initial migration.
    ///
    /// The migrations path is assumed to be `./migrations`.
    ///
    /// # Errors
    ///
    /// Errors if the connection fails or the database cannot be created, or the migration fails.
    pub async fn connect_with_migration(self) -> Result<PgPool, SqlxError> {
        self.connect_with_custom_migration("./migrations").await
    }

    /// Attempts to establish a connection to Postgres and run the initial migration.
    ///
    /// # Errors
    ///
    /// Errors if the migration fails at any point.
    pub async fn connect_with_custom_migration(
        self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<PgPool, SqlxError> {
        let pool = self.connect().await?;
        tracing::info!(
            "running initial migration from '{}'",
            path.as_ref().display()
        );
        let migrator = Migrator::new(path.as_ref()).await?;
        migrator.run(&pool).await?;
        tracing::info!("migration successful");
        Ok(pool)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use sqlx::Row;

    async fn dummy_table(pool: &PgPool) {
        let query = "CREATE TABLE foo(id INT NOT NULL, bar TEXT NOT NULL);";
        pool.execute(query).await.unwrap();
    }

    async fn dummy_text(pool: &PgPool) -> String {
        let query = "INSERT INTO foo (id, bar) VALUES (123, 'hello world');";
        pool.execute(query).await.unwrap();

        let query = "SELECT (bar) FROM foo WHERE id = 123;";
        sqlx::query(query).fetch_one(pool).await.unwrap().get(0)
    }

    async fn drop_table(pool: &PgPool) {
        let query = "DROP TABLE foo;";
        pool.execute(query).await.unwrap();
    }

    #[tokio::test]
    async fn connect_to_postgres() {
        let config = Config::from_env();
        let pool = config.connect().await.unwrap();

        dummy_table(&pool).await;
        let text = dummy_text(&pool).await;

        assert_eq!(text, "hello world");

        drop_table(&pool).await;
    }

    #[tokio::test]
    async fn connect_to_postgres_with_custom_db_name() {
        let config = Config::from_env().with_database("my_random_db");
        assert_eq!(config.pg_connect_opts.get_database(), Some("my_random_db"));
        let pool = config.connect().await.unwrap();

        dummy_table(&pool).await;
        let text = dummy_text(&pool).await;

        assert_eq!(text, "hello world");

        drop_table(&pool).await;
    }

    #[tokio::test]
    async fn connect_to_postgres_with_migration() {
        let config = Config::from_env().with_database("with_migration");
        let pool = config.connect_with_migration().await.unwrap();

        let text = dummy_text(&pool).await;

        assert_eq!(text, "hello world");
    }
}
