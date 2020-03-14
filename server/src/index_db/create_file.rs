use crate::index_db::generate_version::generate_version;
use crate::index_db::generate_version::Error as VersionGenerationError;
use postgres::Client as PostgresClient;
use tokio_postgres;
use tokio_postgres::error::Error as PostgresError;
use tokio_postgres::error::SqlState;

#[derive(Debug)]
pub enum Error {
    FileIdTaken,
    FilePathTaken,
    Uninterpreted(PostgresError),
    VersionGeneration(VersionGenerationError),
}

impl From<PostgresError> for Error {
    fn from(e: PostgresError) -> Error {
        match (e.code(), e.to_string()) {
            (Some(error_code), error_string)
                if error_code == &SqlState::UNIQUE_VIOLATION
                    && error_string.contains("pk_files") =>
            {
                Error::FileIdTaken
            }
            (Some(error_code), error_string)
                if error_code == &SqlState::UNIQUE_VIOLATION
                    && error_string.contains("unique_file_path") =>
            {
                Error::FilePathTaken
            }
            _ => Error::Uninterpreted(e),
        }
    }
}

impl From<VersionGenerationError> for Error {
    fn from(e: VersionGenerationError) -> Error {
        Error::VersionGeneration(e)
    }
}

pub fn create_file(
    client: &mut PostgresClient,
    file_id: &String,
    username: &String,
    file_name: &String,
    file_path: &String,
) -> Result<i64, Error> {
    let version = generate_version(client)?;

    client.execute("
INSERT INTO files (file_id, file_name, file_path, username, file_content_version, file_metadata_version, deleted)
VALUES ($1, $2, $3, $4, $5, $6, $7);
", &[file_id, file_name, file_path, username, &version, &version, &false])?;

    Ok(version)
}