mod options;

pub use options::Options;

use sqlx::migrate::Migrator;
use sqlx::{Error as SqlxError, PgPool};

/// Postgres specific database configuration parameters.
#[derive(Clone, Debug)]
pub struct Config {
    pub options: Options,
    pub migrations_path: String,
}

impl Default for Config {
    fn default() -> Self {
        let migrations_path = "./migrations".to_string();

        Self {
            options: Options::default(),
            migrations_path,
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

        config.options = Options::from_env();

        if let Ok(migrations) = dotenv::var("DB_MIGRATIONS_PATH") {
            config.migrations_path = migrations;
        }

        config
    }

    /// Attempts to establish a connection to Postgres and run the initial migration.
    ///
    /// The default migrations path is assumed to be `./migrations`.
    ///
    /// # Errors
    ///
    /// Errors if the connection fails or the database cannot be created, or the migration fails.
    pub async fn connect_with_migration(self) -> Result<PgPool, SqlxError> {
        tracing::info!("connecting to database");
        let pool = self.options.connect().await?;
        tracing::info!("running initial migration from '{}'", self.migrations_path);
        let migrator = Migrator::new(self.migrations_path.as_ref()).await?;
        migrator.run(&pool).await?;
        tracing::info!("migration successful");
        Ok(pool)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use sqlx::{Executor, Row};

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
        let pool = Config::from_env().options.connect().await.unwrap();

        dummy_table(&pool).await;
        let text = dummy_text(&pool).await;

        assert_eq!(text, "hello world");

        drop_table(&pool).await;
    }

    #[tokio::test]
    async fn connect_to_postgres_with_custom_db_name() {
        let pool = Config::from_env()
            .options
            .with_database("my_random_db")
            .connect()
            .await
            .unwrap();

        dummy_table(&pool).await;
        let text = dummy_text(&pool).await;

        assert_eq!(text, "hello world");

        drop_table(&pool).await;
    }

    #[tokio::test]
    async fn connect_to_postgres_with_migration() {
        let mut config = Config::from_env();
        config.options = config.options.with_database("with_migration");
        let pool = config.connect_with_migration().await.unwrap();

        let text = dummy_text(&pool).await;

        assert_eq!(text, "hello world");
    }
}
