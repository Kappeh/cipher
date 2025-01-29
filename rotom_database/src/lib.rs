#[cfg(feature = "mysql")]
pub mod mysql;
#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "sqlite")]
pub mod sqlite;

#[derive(Clone, Debug)]
pub enum DatabaseDialect {
    #[cfg(feature = "mysql")]
    Mysql,
    #[cfg(feature = "postgres")]
    Postgres,
    #[cfg(feature = "sqlite")]
    Sqlite,
}

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error(transparent)]
    DieselConnectionError(#[from] diesel::ConnectionError),
    #[error(transparent)]
    DieselQueryError(#[from] diesel::result::Error),
    #[error(transparent)]
    DieselMigrationError(Box<dyn std::error::Error + Send + Sync>),
    #[error(transparent)]
    Bb8RunError(#[from] diesel_async::pooled_connection::bb8::RunError),
}

impl From<diesel_async::pooled_connection::PoolError> for BackendError {
    fn from(value: diesel_async::pooled_connection::PoolError) -> Self {
        use diesel_async::pooled_connection::PoolError as E;
        match value {
            E::ConnectionError(connection_error) => Self::from(connection_error),
            E::QueryError(error) => Self::from(error),
        }
    }
}
