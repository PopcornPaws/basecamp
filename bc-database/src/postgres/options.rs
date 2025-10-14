use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use sqlx::{ConnectOptions, Error as SqlxError, Executor, PgPool};
use tracing::log::LevelFilter;

use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Options {
    pub connect: PgConnectOptions,
    pub pool: PgPoolOptions,
}

impl Default for Options {
    fn default() -> Self {
        let connect = PgConnectOptions::new()
            .port(5432)
            .host("localhost")
            .database("postgres")
            .username("postgres")
            .password("password")
            .ssl_mode(PgSslMode::Prefer)
            .log_statements(LevelFilter::Trace);
        let pool = PgPoolOptions::new()
            .acquire_timeout(Duration::from_secs(5))
            .idle_timeout(Duration::from_secs(2));

        Self { connect, pool }
    }
}

impl Options {
    /// Attempts to read database config from environment variables.
    ///
    /// If an environment variable is not set, the respective config is set to its default value.
    ///
    /// # Panics
    ///
    /// Panics if parameters cannot be parsed or if the database url is provided in the wrong
    #[must_use]
    pub fn from_env() -> Self {
        let mut opts = Self::default();

        dotenvy::dotenv().ok();
        // try to parse the whole database url if present
        if let Ok(url) = dotenvy::var("DATABASE_URL") {
            opts.connect = url.parse().expect("invalid database url");
        } else {
            if let Ok(port) = dotenvy::var("DB_PORT") {
                opts.connect = opts.connect.port(port.parse().unwrap());
            }
            if let Ok(host) = dotenvy::var("DB_HOST") {
                opts.connect = opts.connect.host(&host);
            }
            if let Ok(name) = dotenvy::var("DB_NAME") {
                opts.connect = opts.connect.database(&name);
            }
            if let Ok(username) = dotenvy::var("DB_USERNAME") {
                opts.connect = opts.connect.username(&username);
            }
            if let Ok(password) = dotenvy::var("DB_PASSWORD") {
                opts.connect = opts.connect.password(&password);
            }
        }
        if let Ok(s) = dotenvy::var("DB_REQUIRE_SSL")
            && s.parse().unwrap()
        {
            opts.connect = opts.connect.ssl_mode(PgSslMode::Require);
        }
        if let Ok(log_level) = dotenvy::var("DB_LOG_LEVEL") {
            opts.connect = opts.connect.log_statements(log_level.parse().unwrap());
        }
        if let Ok(seconds) = dotenvy::var("DB_ACQUIRE_TIMEOUT") {
            let duration = Duration::from_secs(seconds.parse().unwrap());
            opts.pool = opts.pool.acquire_timeout(duration);
        }
        if let Ok(seconds) = dotenvy::var("DB_IDLE_TIMEOUT") {
            let duration = Duration::from_secs(seconds.parse().unwrap());
            opts.pool = opts.pool.idle_timeout(duration);
        }

        opts
    }

    #[must_use]
    pub fn with_database(self, db: &str) -> Self {
        Self {
            connect: self.connect.database(db),
            pool: self.pool,
        }
    }

    fn connect_lazy_with(self) -> PgPool {
        self.pool.connect_lazy_with(self.connect)
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
        let db = self
            .connect
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
}
