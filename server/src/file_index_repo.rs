use crate::config::IndexDbConfig;
use lockbook_core::model::account::Username;
use lockbook_core::model::crypto::{FolderAccessInfo, SignedValue, UserAccessInfo};
use lockbook_core::model::file_metadata::FileMetadata;
use lockbook_core::model::file_metadata::FileType;
use openssl::error::ErrorStack as OpenSslError;
use openssl::ssl::{SslConnector, SslMethod};
use postgres_openssl::MakeTlsConnector;
use rsa::RSAPublicKey;
use std::collections::HashMap;
use tokio_postgres::error::Error as PostgresError;
use tokio_postgres::error::SqlState;
use tokio_postgres::Client as PostgresClient;
use tokio_postgres::Config as PostgresConfig;
use tokio_postgres::NoTls;
use tokio_postgres::Transaction;
use uuid::Uuid;

// TODO:
// * check ownership
// * better serialization

#[derive(Debug)]
pub enum ConnectError {
    OpenSsl(OpenSslError),
    Postgres(PostgresError),
}

#[derive(Debug)]
pub enum AccountError {
    Postgres(PostgresError),
    Serialization(serde_json::Error),
    UsernameTaken,
}

impl From<PostgresError> for AccountError {
    fn from(e: PostgresError) -> AccountError {
        match e.code() {
            Some(x) if x == &SqlState::UNIQUE_VIOLATION => AccountError::UsernameTaken,
            _ => AccountError::Postgres(e),
        }
    }
}

#[derive(Debug)]
pub enum PublicKeyError {
    UserNotFound,
    Deserialization(serde_json::Error),
    Postgres(PostgresError),
    Unknown(String),
}

#[derive(Debug)]
pub enum FileError {
    Deleted,
    Deserialize(serde_json::Error),
    DoesNotExist,
    IdTaken,
    IncorrectOldVersion,
    OwnerDoesNotExist,
    ParentDoesNotExist,
    PathTaken,
    Postgres(PostgresError),
    Serialize(serde_json::Error),
    WrongFileType,
    Unknown(String),
}

impl From<PostgresError> for FileError {
    fn from(e: PostgresError) -> FileError {
        match (e.code(), e.to_string()) {
            (Some(error_code), error_string)
                if error_code == &SqlState::UNIQUE_VIOLATION
                    && error_string.contains("pk_files") =>
            {
                FileError::IdTaken
            }
            (Some(error_code), error_string)
                if error_code == &SqlState::UNIQUE_VIOLATION
                    && error_string.contains("uk_files_name_parent") =>
            {
                FileError::PathTaken
            }
            (Some(error_code), error_string)
                if error_code == &SqlState::FOREIGN_KEY_VIOLATION
                    && error_string.contains("fk_files_parent_files_id") =>
            {
                FileError::ParentDoesNotExist
            }
            (Some(error_code), error_string)
                if error_code == &SqlState::FOREIGN_KEY_VIOLATION
                    && error_string.contains("fk_files_owner_accounts_name") =>
            {
                FileError::OwnerDoesNotExist
            }
            _ => FileError::Postgres(e),
        }
    }
}

pub async fn connect(config: &IndexDbConfig) -> Result<PostgresClient, ConnectError> {
    let mut postgres_config = PostgresConfig::new();
    postgres_config
        .user(&config.user)
        .host(&config.host)
        .password(&config.pass)
        .port(config.port)
        .dbname(&config.db);

    match config.cert.as_str() {
        "" => connect_no_tls(&postgres_config).await,
        cert => connect_with_tls(&postgres_config, &cert).await,
    }
}

async fn connect_no_tls(postgres_config: &PostgresConfig) -> Result<PostgresClient, ConnectError> {
    match postgres_config.connect(NoTls).await {
        Ok((client, connection)) => {
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    panic!("connection error: {}", e);
                }
            });
            Ok(client)
        }
        Err(err) => Err(ConnectError::Postgres(err)),
    }
}

async fn connect_with_tls(
    postgres_config: &PostgresConfig,
    cert: &str,
) -> Result<PostgresClient, ConnectError> {
    let mut builder = match SslConnector::builder(SslMethod::tls()) {
        Ok(builder) => builder,
        Err(err) => return Err(ConnectError::OpenSsl(err)),
    };
    builder.set_ca_file(cert).map_err(ConnectError::OpenSsl)?;
    match postgres_config
        .connect(MakeTlsConnector::new(builder.build()))
        .await
    {
        Ok((client, connection)) => {
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    panic!("connection error: {}", e);
                }
            });
            Ok(client)
        }
        Err(err) => Err(ConnectError::Postgres(err)),
    }
}

pub async fn change_document_content_version(
    transaction: &Transaction<'_>,
    id: Uuid,
    old_metadata_version: u64,
) -> Result<(u64, u64), FileError> {
    let rows = transaction
        .query(
            "WITH old AS (SELECT * FROM files WHERE id = $1 FOR UPDATE)
            UPDATE files new
            SET
                metadata_version =
                    (CASE WHEN NOT old.deleted AND old.metadata_version = $2 AND NOT old.is_folder
                    THEN CAST(EXTRACT(EPOCH FROM NOW()) * 1000 AS BIGINT)
                    ELSE old.metadata_version END),
                content_version =
                    (CASE WHEN NOT old.deleted AND old.metadata_version = $2 AND NOT old.is_folder
                    THEN CAST(EXTRACT(EPOCH FROM NOW()) * 1000 AS BIGINT)
                    ELSE old.content_version END)
            FROM old WHERE old.id = new.id
            RETURNING
                old.deleted AS old_deleted,
                old.metadata_version AS old_metadata_version,
                old.content_version AS old_content_version,
                new.metadata_version AS new_metadata_version,
                old.is_folder AS is_folder;",
            &[
                &serde_json::to_string(&id).map_err(FileError::Serialize)?,
                &(old_metadata_version as i64),
            ],
        )
        .await
        .map_err(FileError::Postgres)?;
    let metadata = FileUpdateResponse::from_row(rows_to_row(&rows)?)?
        .validate(old_metadata_version, FileType::Document)?;
    Ok((metadata.old_content_version, metadata.new_metadata_version))
}

pub async fn create_file(
    transaction: &Transaction<'_>,
    id: Uuid,
    parent: Uuid,
    file_type: FileType,
    name: &str,
    owner: &str,
    signature: &SignedValue,
    access_key: &FolderAccessInfo,
) -> Result<u64, FileError> {
    let row = transaction
        .query_one(
            "INSERT INTO files (id, parent, parent_access_key, is_folder, name, owner, signature, deleted, metadata_version, content_version)
            VALUES ($1, $2, $3, $4, $5, $6, $7, FALSE, CAST(EXTRACT(EPOCH FROM NOW()) * 1000 AS BIGINT), CAST(EXTRACT(EPOCH FROM NOW()) * 1000 AS BIGINT))
            RETURNING metadata_version;",
            &[
                &serde_json::to_string(&id).map_err(FileError::Serialize)?,
                &serde_json::to_string(&parent).map_err(FileError::Serialize)?,
                &serde_json::to_string(&access_key)
                    .map_err(FileError::Serialize)?,
                &(file_type == FileType::Folder),
                &name,
                &owner,
                &serde_json::to_string(&signature).map_err(FileError::Serialize)?,
            ],
        )
        .await?;
    Ok(row
        .try_get::<&str, i64>("metadata_version")
        .map_err(FileError::Postgres)? as u64)
}

pub async fn delete_file(
    transaction: &Transaction<'_>,
    id: Uuid,
    old_metadata_version: u64,
    file_type: FileType,
) -> Result<(u64, u64), FileError> {
    let rows = transaction
        .query(
            "WITH old AS (SELECT * FROM files WHERE id = $1 FOR UPDATE)
            UPDATE files new
            SET
                deleted =
                    (CASE WHEN NOT old.deleted AND old.metadata_version = $2 AND old.is_folder = $3
                    THEN TRUE
                    ELSE old.deleted END),
                metadata_version =
                    (CASE WHEN NOT old.deleted AND old.metadata_version = $2 AND old.is_folder = $3
                    THEN CAST(EXTRACT(EPOCH FROM NOW()) * 1000 AS BIGINT)
                    ELSE old.metadata_version END)
            FROM old WHERE old.id = new.id
            RETURNING
                old.deleted AS old_deleted,
                old.metadata_version AS old_metadata_version,
                old.content_version AS old_content_version,
                new.metadata_version AS new_metadata_version,
                old.is_folder AS is_folder;",
            &[
                &serde_json::to_string(&id).map_err(FileError::Serialize)?,
                &(old_metadata_version as i64),
                &(file_type == FileType::Folder),
            ],
        )
        .await
        .map_err(FileError::Postgres)?;
    let metadata = FileUpdateResponse::from_row(rows_to_row(&rows)?)?
        .validate(old_metadata_version, file_type)?;
    Ok((metadata.old_content_version, metadata.new_metadata_version))
}

pub async fn move_file(
    transaction: &Transaction<'_>,
    id: Uuid,
    old_metadata_version: u64,
    file_type: FileType,
    parent: Uuid,
    access_key: FolderAccessInfo,
) -> Result<u64, FileError> {
    let rows = transaction
        .query(
            "WITH old AS (SELECT * FROM files WHERE id = $1 FOR UPDATE)
            UPDATE files new
            SET
                parent =
                    (CASE WHEN NOT old.deleted AND old.metadata_version = $2 AND old.is_folder = $3
                    THEN $4
                    ELSE old.parent END),
                metadata_version =
                    (CASE WHEN NOT old.deleted AND old.metadata_version = $2 AND old.is_folder = $3
                    THEN CAST(EXTRACT(EPOCH FROM NOW()) * 1000 AS BIGINT)
                    ELSE old.metadata_version END),
                parent_access_key =
                    (CASE WHEN NOT old.deleted AND old.metadata_version = $2 AND old.is_folder = $3
                    THEN $5
                    ELSE old.parent_access_key END)
            FROM old WHERE old.id = new.id
            RETURNING
                old.deleted AS old_deleted,
                old.metadata_version AS old_metadata_version,
                old.content_version AS old_content_version,
                new.metadata_version AS new_metadata_version,
                old.is_folder AS is_folder;",
            &[
                &serde_json::to_string(&id).map_err(FileError::Serialize)?,
                &(old_metadata_version as i64),
                &(file_type == FileType::Folder),
                &serde_json::to_string(&parent).map_err(FileError::Serialize)?,
                &serde_json::to_string(&access_key).map_err(FileError::Serialize)?,
            ],
        )
        .await?;
    let metadata = FileUpdateResponse::from_row(rows_to_row(&rows)?)?
        .validate(old_metadata_version, file_type)?;
    Ok(metadata.new_metadata_version)
}

pub async fn rename_file(
    transaction: &Transaction<'_>,
    id: Uuid,
    old_metadata_version: u64,
    file_type: FileType,
    name: &str,
) -> Result<u64, FileError> {
    let rows = transaction
        .query(
            "WITH old AS (SELECT * FROM files WHERE id = $1 FOR UPDATE)
            UPDATE files new
            SET
                name =
                    (CASE WHEN NOT old.deleted AND old.metadata_version = $2 AND old.is_folder = $3
                    THEN $4
                    ELSE old.name END),
                metadata_version =
                    (CASE WHEN NOT old.deleted AND old.metadata_version = $2 AND old.is_folder = $3
                    THEN CAST(EXTRACT(EPOCH FROM NOW()) * 1000 AS BIGINT)
                    ELSE old.metadata_version END)
            FROM old WHERE old.id = new.id
            RETURNING
                old.deleted AS old_deleted,
                old.metadata_version AS old_metadata_version,
                old.content_version AS old_content_version,
                new.metadata_version AS new_metadata_version,
                old.is_folder AS is_folder;",
            &[
                &serde_json::to_string(&id).map_err(FileError::Serialize)?,
                &(old_metadata_version as i64),
                &(file_type == FileType::Folder),
                &name,
            ],
        )
        .await?;
    let metadata = FileUpdateResponse::from_row(rows_to_row(&rows)?)?
        .validate(old_metadata_version, file_type)?;
    Ok(metadata.new_metadata_version)
}

pub async fn get_public_key(
    transaction: &Transaction<'_>,
    username: &str,
) -> Result<RSAPublicKey, PublicKeyError> {
    match transaction
        .query(
            "SELECT public_key FROM accounts WHERE name = $1;",
            &[&username],
        )
        .await
        .map_err(PublicKeyError::Postgres)?
        .as_slice()
    {
        [] => Err(PublicKeyError::UserNotFound),
        [row] => {
            Ok(serde_json::from_str(row.get("public_key"))
                .map_err(PublicKeyError::Deserialization)?)
        }
        _ => Err(PublicKeyError::Unknown(String::from(
            "unexpected multiple postgres rows",
        ))),
    }
}

fn rows_to_row<'a>(
    rows: &'a Vec<tokio_postgres::row::Row>,
) -> Result<&'a tokio_postgres::row::Row, FileError> {
    match rows.as_slice() {
        [] => Err(FileError::DoesNotExist),
        [row] => Ok(row),
        _ => Err(FileError::Unknown(String::from(
            "unexpected multiple postgres rows",
        ))),
    }
}

struct FileUpdateResponse {
    old_deleted: bool,
    old_metadata_version: u64,
    old_content_version: u64,
    new_metadata_version: u64,
    is_folder: bool,
}

impl FileUpdateResponse {
    fn from_row(row: &tokio_postgres::row::Row) -> Result<FileUpdateResponse, FileError> {
        Ok(FileUpdateResponse {
            old_deleted: row.try_get("old_deleted").map_err(FileError::Postgres)?,
            old_metadata_version: row
                .try_get::<&str, i64>("old_metadata_version")
                .map_err(FileError::Postgres)? as u64,
            old_content_version: row
                .try_get::<&str, i64>("old_content_version")
                .map_err(FileError::Postgres)? as u64,
            new_metadata_version: row
                .try_get::<&str, i64>("new_metadata_version")
                .map_err(FileError::Postgres)? as u64,
            is_folder: row.try_get("is_folder").map_err(FileError::Postgres)?,
        })
    }

    fn validate(
        self,
        expected_old_metadata_version: u64,
        expected_file_type: FileType,
    ) -> Result<FileUpdateResponse, FileError> {
        if self.is_folder != (expected_file_type == FileType::Folder) {
            Err(FileError::WrongFileType)
        } else if self.old_deleted {
            Err(FileError::Deleted)
        } else if self.old_metadata_version != expected_old_metadata_version {
            Err(FileError::IncorrectOldVersion)
        } else {
            Ok(self)
        }
    }
}

fn row_to_file_metadata(row: &tokio_postgres::row::Row) -> Result<FileMetadata, FileError> {
    Ok(FileMetadata {
        id: serde_json::from_str(
            row.try_get::<&str, &str>("id")
                .map_err(FileError::Postgres)?,
        )
        .map_err(FileError::Deserialize)?,
        file_type: {
            if row
                .try_get::<&str, bool>("is_folder")
                .map_err(FileError::Postgres)?
            {
                FileType::Folder
            } else {
                FileType::Document
            }
        },
        parent: serde_json::from_str(
            row.try_get::<&str, &str>("parent")
                .map_err(FileError::Postgres)?,
        )
        .map_err(FileError::Deserialize)?,
        name: row.try_get("name").map_err(FileError::Postgres)?,
        owner: row.try_get("owner").map_err(FileError::Postgres)?,
        signature: serde_json::from_str(
            row.try_get::<&str, &str>("signature")
                .map_err(FileError::Postgres)?,
        )
        .map_err(FileError::Deserialize)?,
        metadata_version: row
            .try_get::<&str, i64>("metadata_version")
            .map_err(FileError::Postgres)? as u64,
        content_version: row
            .try_get::<&str, i64>("content_version")
            .map_err(FileError::Postgres)? as u64,
        deleted: row.try_get("deleted").map_err(FileError::Postgres)?,
        user_access_keys: {
            let username: Username = row.try_get("name").map_err(FileError::Postgres)?;
            let encrypted_key_res = row.try_get::<&str, &str>("encrypted_key");
            let public_key_res = row.try_get::<&str, &str>("public_key");

            let mut user_access_keys: HashMap<Username, UserAccessInfo> = HashMap::new();

            if let (Ok(encrypted_key), Ok(public_key)) = (encrypted_key_res, public_key_res) {
                user_access_keys.insert(
                    username.clone(),
                    UserAccessInfo {
                        username,
                        public_key: serde_json::from_str(public_key)
                            .map_err(FileError::Deserialize)?,
                        access_key: serde_json::from_str(encrypted_key)
                            .map_err(FileError::Deserialize)?,
                    },
                );
            };
            user_access_keys
        },
        folder_access_keys: serde_json::from_str(
            row.try_get::<&str, &str>("parent_access_key")
                .map_err(FileError::Postgres)?,
        )
        .map_err(FileError::Deserialize)?,
    })
}

pub async fn get_updates(
    transaction: &Transaction<'_>,
    username: &str,
    metadata_version: u64,
) -> Result<Vec<FileMetadata>, FileError> {
    transaction
        .query(
            "SELECT * FROM files fi
                        LEFT JOIN user_access_keys uak ON fi.id = uak.file_id AND fi.owner = uak.sharee_id
                        LEFT JOIN accounts a ON fi.owner = a.name
                        WHERE owner = $1
                        AND metadata_version > $2;",
            &[&username, &(metadata_version as i64)],
        )
        .await
        .map_err(FileError::Postgres)?
        .iter()
        .map(row_to_file_metadata)
        .collect()
}

pub async fn new_account(
    transaction: &Transaction<'_>,
    username: &str,
    public_key: &str,
) -> Result<(), AccountError> {
    transaction
        .execute(
            "INSERT INTO accounts (name, public_key) VALUES ($1, $2);",
            &[&username, &public_key],
        )
        .await?;
    Ok(())
}

pub async fn create_user_access_key(
    transaction: &Transaction<'_>,
    username: &str,
    folder_id: Uuid,
    user_access_key: &str,
) -> Result<(), AccountError> {
    transaction
        .execute(
            "INSERT INTO user_access_keys (file_id, sharee_id, encrypted_key) VALUES ($1, $2, $3);",
            &[
                &serde_json::to_string(&folder_id).map_err(AccountError::Serialization)?,
                &username,
                &user_access_key,
            ],
        )
        .await?;
    Ok(())
}