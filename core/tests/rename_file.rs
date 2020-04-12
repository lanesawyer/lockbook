extern crate lockbook_core;
use lockbook_core::lockbook_api;
use lockbook_core::lockbook_api::CreateFileRequest;
use lockbook_core::lockbook_api::DeleteFileRequest;
use lockbook_core::lockbook_api::NewAccountRequest;
use lockbook_core::lockbook_api::{RenameFileError, RenameFileRequest};

#[macro_use]
pub mod utils;
use utils::{api_loc, generate_file_id, generate_username, TestError};

fn rename_file() -> Result<(), TestError> {
    let username = generate_username();
    let file_id = generate_file_id();

    lockbook_api::new_account(
        api_loc(),
        &NewAccountRequest {
            username: username.to_string(),
            auth: "test_auth".to_string(),
            pub_key_n: "test_pub_key_n".to_string(),
            pub_key_e: "test_pub_key_e".to_string(),
        },
    )?;

    lockbook_api::create_file(
        api_loc(),
        &CreateFileRequest {
            username: username.to_string(),
            auth: "test_auth".to_string(),
            file_id: file_id.to_string(),
            file_name: "file_name".to_string(),
            file_path: "file_path".to_string(),
            file_content: "file_content".to_string(),
        },
    )?;

    lockbook_api::rename_file(
        api_loc(),
        &RenameFileRequest {
            username: username.to_string(),
            auth: "test_auth".to_string(),
            file_id: file_id.to_string(),
            new_file_name: "new_file_name".to_string(),
        },
    )?;

    Ok(())
}

#[test]
fn test_rename_file() {
    assert_matches!(rename_file(), Ok(_));
}

fn rename_file_file_not_found() -> Result<(), TestError> {
    let username = generate_username();

    lockbook_api::new_account(
        api_loc(),
        &NewAccountRequest {
            username: username.to_string(),
            auth: "test_auth".to_string(),
            pub_key_n: "test_pub_key_n".to_string(),
            pub_key_e: "test_pub_key_e".to_string(),
        },
    )?;

    lockbook_api::rename_file(
        api_loc(),
        &RenameFileRequest {
            username: username.to_string(),
            auth: "test_auth".to_string(),
            file_id: generate_file_id(),
            new_file_name: "new_file_name".to_string(),
        },
    )?;

    Ok(())
}

#[test]
fn test_rename_file_file_not_found() {
    assert_matches!(
        rename_file_file_not_found(),
        Err(TestError::RenameFileError(RenameFileError::FileNotFound))
    );
}

fn rename_file_file_deleted() -> Result<(), TestError> {
    let username = generate_username();
    let file_id = generate_file_id();

    lockbook_api::new_account(
        api_loc(),
        &NewAccountRequest {
            username: username.to_string(),
            auth: "test_auth".to_string(),
            pub_key_n: "test_pub_key_n".to_string(),
            pub_key_e: "test_pub_key_e".to_string(),
        },
    )?;

    lockbook_api::create_file(
        api_loc(),
        &CreateFileRequest {
            username: username.to_string(),
            auth: "test_auth".to_string(),
            file_id: file_id.to_string(),
            file_name: "file_name".to_string(),
            file_path: "file_path".to_string(),
            file_content: "file_content".to_string(),
        },
    )?;

    lockbook_api::delete_file(
        api_loc(),
        &DeleteFileRequest {
            username: username.to_string(),
            auth: "test_auth".to_string(),
            file_id: file_id.to_string(),
        },
    )?;

    lockbook_api::rename_file(
        api_loc(),
        &RenameFileRequest {
            username: username.to_string(),
            auth: "test_auth".to_string(),
            file_id: file_id.to_string(),
            new_file_name: "new_file_name".to_string(),
        },
    )?;

    Ok(())
}

#[test]
fn test_rename_file_file_deleted() {
    assert_matches!(
        rename_file_file_deleted(),
        Err(TestError::RenameFileError(RenameFileError::FileDeleted))
    );
}