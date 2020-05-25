use crate::utils::{connect_to_db, edit_file_with_editor, get_account, get_editor};
use lockbook_core::repo::file_metadata_repo::FileMetadataRepo;
use lockbook_core::service::file_service::FileService;
use lockbook_core::service::sync_service::SyncService;
use lockbook_core::{DefaultFileMetadataRepo, DefaultFileService, DefaultSyncService};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::{fs, io};
use uuid::Uuid;

pub fn edit() {
    let db = connect_to_db();
    get_account(&db);

    let file_location = format!("/tmp/{}", Uuid::new_v4().to_string());
    let temp_file_path = Path::new(file_location.as_str());
    let mut file_handle = File::create(&temp_file_path)
        .expect(format!("Could not create temporary file: {}", &file_location).as_str());

    print!("Enter a filename: ");
    io::stdout().flush().unwrap();
    let mut file_name = String::new();
    io::stdin()
        .read_line(&mut file_name)
        .expect("Failed to read from stdin");
    file_name.retain(|c| !c.is_whitespace());

    let mut file_metadata = DefaultFileMetadataRepo::find_by_name(&db, &file_name)
        .expect("Could not search files ")
        .expect("Could not find that file!");

    let file_content = match DefaultFileService::get(&db, &file_metadata.file_id) {
        Ok(content) => content,
        Err(error) => panic!("Unexpected error: {:?}", error),
    };

    file_handle
        .write_all(&file_content.secret.into_bytes())
        .expect(
            format!(
                "Failed to write decrypted contents to temporary file, check {}",
                file_location
            )
            .as_str(),
        );
    file_handle.sync_all().expect(
        format!(
            "Failed to write decrypted contents to temporary file, check {}",
            file_location
        )
        .as_str(),
    );

    let edit_was_successful = edit_file_with_editor(&file_location);

    if edit_was_successful {
        let file_content =
            fs::read_to_string(temp_file_path).expect("Could not read file that was edited");

        DefaultFileService::update(&db, &file_metadata.file_id, &file_content)
            .expect("Unexpected error while updating internal state");

        file_metadata.content_edited_locally = true;

        println!("Updating local state.");
        DefaultFileMetadataRepo::update(&db, &file_metadata).expect("Failed to index new file!");

        println!("Syncing");
        DefaultSyncService::sync(&db).expect("Failed to sync");

        println!("Sync successful, cleaning up.")
    } else {
        eprintln!(
            "{} indicated a problem, aborting and cleaning up",
            get_editor()
        );
    }

    fs::remove_file(&temp_file_path)
        .expect(format!("Failed to delete temporary file: {}", &file_location).as_str());
}