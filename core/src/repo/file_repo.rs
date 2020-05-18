use std::option::NoneError;

use crate::error_enum;
use crate::service::file_encryption_service::EncryptedFile;
use serde_json;
use sled;
use sled::Db;

error_enum! {
    enum Error {
        SledError(sled::Error),
        SerdeError(serde_json::Error),
        FileRowMissing(NoneError)
    }
}

pub trait FileRepo {
    fn update(db: &Db, id: &String, file: &EncryptedFile) -> Result<(), Error>;
    fn get(db: &Db, id: &String) -> Result<EncryptedFile, Error>;
    fn delete(db: &Db, id: &String) -> Result<(), Error>;
}

pub struct FileRepoImpl;

impl FileRepo for FileRepoImpl {
    fn update(db: &Db, id: &String, file: &EncryptedFile) -> Result<(), Error> {
        let tree = db.open_tree(b"files")?;
        tree.insert(id.as_bytes(), serde_json::to_vec(file)?)?;
        Ok(())
    }

    fn get(db: &Db, id: &String) -> Result<EncryptedFile, Error> {
        let tree = db.open_tree(b"files")?;
        let maybe_value = tree.get(id.as_bytes())?;
        let value = maybe_value?;
        let file: EncryptedFile = serde_json::from_slice(value.as_ref())?;

        Ok(file)
    }

    fn delete(db: &Db, id: &String) -> Result<(), Error> {
        let tree = db.open_tree(b"files")?;
        tree.remove(id.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod unit_tests {
    use crate::model::state::Config;
    use crate::repo::db_provider::{DbProvider, TempBackedDB};
    use crate::repo::file_repo::{FileRepo, FileRepoImpl};
    use crate::service::crypto_service::{EncryptedValueWithNonce, SignedValue};
    use crate::service::file_encryption_service::EncryptedFile;

    type DefaultDbProvider = TempBackedDB;

    #[test]
    fn update_file() {
        let test_file = EncryptedFile {
            access_keys: Default::default(),
            content: EncryptedValueWithNonce {
                garbage: "something".to_string(),
                nonce: "nonce1".to_string(),
            },
            last_edited: SignedValue {
                content: "".to_string(),
                signature: "".to_string(),
            },
        };

        let config = Config {
            writeable_path: "ignored".to_string(),
        };
        let db = DefaultDbProvider::connect_to_db(&config).unwrap();
        let file_id = &"a".to_string();

        FileRepoImpl::update(&db, file_id, &test_file).unwrap();

        let file = FileRepoImpl::get(&db, &"a".to_string()).unwrap();
        assert_eq!(
            file.content,
            EncryptedValueWithNonce {
                garbage: "something".to_string(),
                nonce: "nonce1".to_string()
            }
        );

        FileRepoImpl::update(
            &db,
            file_id,
            &EncryptedFile {
                access_keys: Default::default(),
                content: EncryptedValueWithNonce {
                    garbage: "updated".to_string(),
                    nonce: "nonce2".to_string(),
                },
                last_edited: SignedValue {
                    content: "".to_string(),
                    signature: "".to_string(),
                },
            },
        )
        .unwrap();

        let file_updated = FileRepoImpl::get(&db, file_id).unwrap();

        assert_eq!(
            file_updated.content,
            EncryptedValueWithNonce {
                garbage: "updated".to_string(),
                nonce: "nonce2".to_string(),
            }
        );
    }
}