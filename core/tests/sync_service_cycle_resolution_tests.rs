use lockbook_core::Core;
use test_utils::*;

/// Tests which are constructed to test cycle resolution. These are tests that setup two synced
/// devices, operate on both devices, then sync both twice (work should be none, devices dbs should
/// be equal, deleted files should be pruned)

fn sync_and_assert_stuff(c1: &Core, c2: &Core) {
    c1.sync(None).unwrap();
    c2.sync(None).unwrap();
    c1.sync(None).unwrap();
    c2.sync(None).unwrap();

    c1.validate().unwrap();
    assert_dbs_eq(c1, c2);
    assert_local_work_paths(c1, &[]);
    assert_server_work_paths(c1, &[]);
    assert_deleted_files_pruned(c1);
}

#[test]
fn two_cycle() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c2, "/b/", "/a/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/b/", "/b/a/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn three_cycle_one_move_reverted() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c1, "/b/", "/c/").unwrap();
    move_by_path(&c2, "/c/", "/a/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/c/", "/c/b/", "/c/b/a/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn three_cycle_two_moves_reverted() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c2, "/b/", "/c/").unwrap();
    move_by_path(&c2, "/c/", "/a/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/b/", "/b/a/", "/c/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn four_cycle_one_move_reverted() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.create_at_path("/d/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c1, "/b/", "/c/").unwrap();
    move_by_path(&c1, "/c/", "/d/").unwrap();
    move_by_path(&c2, "/d/", "/a/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/d/", "/d/c/", "/d/c/b/", "/d/c/b/a/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn four_cycle_two_moves_reverted_adjacent() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.create_at_path("/d/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c1, "/b/", "/c/").unwrap();
    move_by_path(&c2, "/c/", "/d/").unwrap();
    move_by_path(&c2, "/d/", "/a/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/c/", "/c/b/", "/c/b/a/", "/d/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn four_cycle_two_moves_reverted_alternating() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.create_at_path("/d/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c2, "/b/", "/c/").unwrap();
    move_by_path(&c1, "/c/", "/d/").unwrap();
    move_by_path(&c2, "/d/", "/a/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/b/", "/b/a/", "/d/", "/d/c/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn four_cycle_three_moves_reverted() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.create_at_path("/d/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c2, "/b/", "/c/").unwrap();
    move_by_path(&c2, "/c/", "/d/").unwrap();
    move_by_path(&c2, "/d/", "/a/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/b/", "/b/a/", "/c/", "/d/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn two_cycle_with_renames_first_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    rename_path(&c1, "/a/", "a2").unwrap();
    rename_path(&c1, "/b/", "b2").unwrap();

    move_by_path(&c1, "/a2/", "/b2/").unwrap();
    move_by_path(&c2, "/b/", "/a/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/b2/", "/b2/a2/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn three_cycle_one_move_reverted_with_renames_first_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    rename_path(&c1, "/a/", "a2").unwrap();
    rename_path(&c1, "/b/", "b2").unwrap();
    rename_path(&c1, "/c/", "c2").unwrap();

    move_by_path(&c1, "/a2/", "/b2/").unwrap();
    move_by_path(&c1, "/b2/", "/c2/").unwrap();
    move_by_path(&c2, "/c/", "/a/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/c2/", "/c2/b2/", "/c2/b2/a2/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn three_cycle_two_moves_reverted_with_renames_first_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    rename_path(&c1, "/a/", "a2").unwrap();
    rename_path(&c1, "/b/", "b2").unwrap();
    rename_path(&c1, "/c/", "c2").unwrap();

    move_by_path(&c1, "/a2/", "/b2/").unwrap();
    move_by_path(&c2, "/b/", "/c/").unwrap();
    move_by_path(&c2, "/c/", "/a/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/b2/", "/b2/a2/", "/c2/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn four_cycle_one_move_reverted_with_renames_first_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.create_at_path("/d/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    rename_path(&c1, "/a/", "a2").unwrap();
    rename_path(&c1, "/b/", "b2").unwrap();
    rename_path(&c1, "/c/", "c2").unwrap();
    rename_path(&c1, "/d/", "d2").unwrap();

    move_by_path(&c1, "/a2/", "/b2/").unwrap();
    move_by_path(&c1, "/b2/", "/c2/").unwrap();
    move_by_path(&c1, "/c2/", "/d2/").unwrap();
    move_by_path(&c2, "/d/", "/a/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/d2/", "/d2/c2/", "/d2/c2/b2/", "/d2/c2/b2/a2/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn four_cycle_two_moves_reverted_adjacent_with_renames_first_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.create_at_path("/d/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    rename_path(&c1, "/a/", "a2").unwrap();
    rename_path(&c1, "/b/", "b2").unwrap();
    rename_path(&c1, "/c/", "c2").unwrap();
    rename_path(&c1, "/d/", "d2").unwrap();

    move_by_path(&c1, "/a2/", "/b2/").unwrap();
    move_by_path(&c1, "/b2/", "/c2/").unwrap();
    move_by_path(&c2, "/c/", "/d/").unwrap();
    move_by_path(&c2, "/d/", "/a/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/c2/", "/c2/b2/", "/c2/b2/a2/", "/d2/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn four_cycle_two_moves_reverted_alternating_with_renames_first_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.create_at_path("/d/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    rename_path(&c1, "/a/", "a2").unwrap();
    rename_path(&c1, "/b/", "b2").unwrap();
    rename_path(&c1, "/c/", "c2").unwrap();
    rename_path(&c1, "/d/", "d2").unwrap();

    move_by_path(&c1, "/a2/", "/b2/").unwrap();
    move_by_path(&c2, "/b/", "/c/").unwrap();
    move_by_path(&c1, "/c2/", "/d2/").unwrap();
    move_by_path(&c2, "/d/", "/a/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/b2/", "/b2/a2/", "/d2/", "/d2/c2/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn four_cycle_three_moves_reverted_with_renames_first_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.create_at_path("/d/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    rename_path(&c1, "/a/", "a2").unwrap();
    rename_path(&c1, "/b/", "b2").unwrap();
    rename_path(&c1, "/c/", "c2").unwrap();
    rename_path(&c1, "/d/", "d2").unwrap();

    move_by_path(&c1, "/a2/", "/b2/").unwrap();
    move_by_path(&c2, "/b/", "/c/").unwrap();
    move_by_path(&c2, "/c/", "/d/").unwrap();
    move_by_path(&c2, "/d/", "/a/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/b2/", "/b2/a2/", "/c2/", "/d2/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn two_cycle_with_renames_second_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    rename_path(&c2, "/a/", "a2").unwrap();
    rename_path(&c2, "/b/", "b2").unwrap();
    move_by_path(&c2, "/b2/", "/a2/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/b2/", "/b2/a2/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn three_cycle_one_move_reverted_with_renames_second_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c1, "/b/", "/c/").unwrap();
    rename_path(&c2, "/a/", "a2").unwrap();
    rename_path(&c2, "/b/", "b2").unwrap();
    rename_path(&c2, "/c/", "c2").unwrap();
    move_by_path(&c2, "/c2/", "/a2/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/c2/", "/c2/b2/", "/c2/b2/a2/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn three_cycle_two_moves_reverted_with_renames_second_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    rename_path(&c2, "/a/", "a2").unwrap();
    rename_path(&c2, "/b/", "b2").unwrap();
    rename_path(&c2, "/c/", "c2").unwrap();
    move_by_path(&c2, "/b2/", "/c2/").unwrap();
    move_by_path(&c2, "/c2/", "/a2/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/b2/", "/b2/a2/", "/c2/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn four_cycle_one_move_reverted_with_renames_second_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.create_at_path("/d/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c1, "/b/", "/c/").unwrap();
    move_by_path(&c1, "/c/", "/d/").unwrap();

    rename_path(&c2, "/a/", "a2").unwrap();
    rename_path(&c2, "/b/", "b2").unwrap();
    rename_path(&c2, "/c/", "c2").unwrap();
    rename_path(&c2, "/d/", "d2").unwrap();

    move_by_path(&c2, "/d2/", "/a2/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/d2/", "/d2/c2/", "/d2/c2/b2/", "/d2/c2/b2/a2/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn four_cycle_two_moves_reverted_adjacent_with_renames_second_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.create_at_path("/d/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c1, "/b/", "/c/").unwrap();

    rename_path(&c2, "/a/", "a2").unwrap();
    rename_path(&c2, "/b/", "b2").unwrap();
    rename_path(&c2, "/c/", "c2").unwrap();
    rename_path(&c2, "/d/", "d2").unwrap();

    move_by_path(&c2, "/c2/", "/d2/").unwrap();
    move_by_path(&c2, "/d2/", "/a2/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/c2/", "/c2/b2/", "/c2/b2/a2/", "/d2/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn four_cycle_two_moves_reverted_alternating_with_renames_second_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.create_at_path("/d/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();

    rename_path(&c2, "/a/", "a2").unwrap();
    rename_path(&c2, "/b/", "b2").unwrap();
    rename_path(&c2, "/c/", "c2").unwrap();
    rename_path(&c2, "/d/", "d2").unwrap();

    move_by_path(&c2, "/b2/", "/c2/").unwrap();
    move_by_path(&c1, "/c/", "/d/").unwrap();
    move_by_path(&c2, "/d2/", "/a2/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/b2/", "/b2/a2/", "/d2/", "/d2/c2/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn four_cycle_three_moves_reverted_with_renames_second_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.create_at_path("/d/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();

    rename_path(&c2, "/a/", "a2").unwrap();
    rename_path(&c2, "/b/", "b2").unwrap();
    rename_path(&c2, "/c/", "c2").unwrap();
    rename_path(&c2, "/d/", "d2").unwrap();

    move_by_path(&c2, "/b2/", "/c2/").unwrap();
    move_by_path(&c2, "/c2/", "/d2/").unwrap();
    move_by_path(&c2, "/d2/", "/a2/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/b2/", "/b2/a2/", "/c2/", "/d2/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn two_cycle_with_deletes_first_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c2, "/b/", "/a/").unwrap();
    delete_path(&c1, "/b/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn three_cycle_one_move_reverted_with_deletes_first_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c1, "/b/", "/c/").unwrap();
    move_by_path(&c2, "/c/", "/a/").unwrap();
    delete_path(&c1, "/c/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn three_cycle_two_moves_reverted_with_deletes_first_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c2, "/b/", "/c/").unwrap();
    move_by_path(&c2, "/c/", "/a/").unwrap();
    delete_path(&c1, "/b/").unwrap();
    delete_path(&c1, "/c/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn four_cycle_one_move_reverted_with_deletes_first_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.create_at_path("/d/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c1, "/b/", "/c/").unwrap();
    move_by_path(&c1, "/c/", "/d/").unwrap();
    move_by_path(&c2, "/d/", "/a/").unwrap();
    delete_path(&c1, "/d/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn four_cycle_two_moves_reverted_adjacent_with_deletes_first_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.create_at_path("/d/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c1, "/b/", "/c/").unwrap();
    move_by_path(&c2, "/c/", "/d/").unwrap();
    move_by_path(&c2, "/d/", "/a/").unwrap();
    delete_path(&c1, "/c/").unwrap();
    delete_path(&c1, "/d/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn four_cycle_two_moves_reverted_alternating_with_deletes_first_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.create_at_path("/d/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c2, "/b/", "/c/").unwrap();
    move_by_path(&c1, "/c/", "/d/").unwrap();
    move_by_path(&c2, "/d/", "/a/").unwrap();
    delete_path(&c1, "/b/").unwrap();
    delete_path(&c1, "/d/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn four_cycle_three_moves_reverted_with_deletes_first_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.create_at_path("/d/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c2, "/b/", "/c/").unwrap();
    move_by_path(&c2, "/c/", "/d/").unwrap();
    move_by_path(&c2, "/d/", "/a/").unwrap();

    delete_path(&c1, "/b/").unwrap();
    delete_path(&c1, "/c/").unwrap();
    delete_path(&c1, "/d/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn two_cycle_with_deletes_second_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c2, "/b/", "/a/").unwrap();
    delete_path(&c2, "/a/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/b/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn three_cycle_one_move_reverted_with_deletes_second_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c1, "/b/", "/c/").unwrap();
    move_by_path(&c2, "/c/", "/a/").unwrap();
    delete_path(&c2, "/a/").unwrap();
    delete_path(&c2, "/b/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/c/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn three_cycle_two_moves_reverted_with_deletes_second_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c2, "/b/", "/c/").unwrap();
    move_by_path(&c2, "/c/", "/a/").unwrap();
    delete_path(&c2, "/a/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/b/", "/c/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn four_cycle_one_move_reverted_with_deletes_second_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.create_at_path("/d/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c1, "/b/", "/c/").unwrap();
    move_by_path(&c1, "/c/", "/d/").unwrap();
    move_by_path(&c2, "/d/", "/a/").unwrap();

    delete_path(&c2, "/a/").unwrap();
    delete_path(&c2, "/b/").unwrap();
    delete_path(&c2, "/c/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/d/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn four_cycle_two_moves_reverted_adjacent_with_deletes_second_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.create_at_path("/d/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c1, "/b/", "/c/").unwrap();
    move_by_path(&c2, "/c/", "/d/").unwrap();
    move_by_path(&c2, "/d/", "/a/").unwrap();

    delete_path(&c2, "/a/").unwrap();
    delete_path(&c2, "/b/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/c/", "/d/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn four_cycle_two_moves_reverted_alternating_with_deletes_second_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.create_at_path("/d/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c2, "/b/", "/c/").unwrap();
    move_by_path(&c1, "/c/", "/d/").unwrap();
    move_by_path(&c2, "/d/", "/a/").unwrap();

    delete_path(&c2, "/a/").unwrap();
    delete_path(&c2, "/c/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/b/", "/d/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn four_cycle_three_moves_reverted_with_deletes_second_device() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/").unwrap();
    c1.create_at_path("/b/").unwrap();
    c1.create_at_path("/c/").unwrap();
    c1.create_at_path("/d/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c2, "/b/", "/c/").unwrap();
    move_by_path(&c2, "/c/", "/d/").unwrap();
    move_by_path(&c2, "/d/", "/a/").unwrap();
    delete_path(&c2, "/a/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/b/", "/c/", "/d/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn move_two_cycle_with_children() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/child/").unwrap();
    c1.create_at_path("/b/child/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c2, "/b/", "/a/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/b/", "/b/a/", "/b/child/", "/b/a/child/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn three_cycle_one_move_reverted_with_children() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/child/").unwrap();
    c1.create_at_path("/b/child/").unwrap();
    c1.create_at_path("/c/child/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c1, "/b/", "/c/").unwrap();
    move_by_path(&c2, "/c/", "/a/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(
        &c2,
        &["/", "/c/", "/c/b/", "/c/b/a/", "/c/child/", "/c/b/child/", "/c/b/a/child/"],
    );
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn three_cycle_two_moves_reverted_with_children() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/child/").unwrap();
    c1.create_at_path("/b/child/").unwrap();
    c1.create_at_path("/c/child/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c2, "/b/", "/c/").unwrap();
    move_by_path(&c2, "/c/", "/a/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(&c2, &["/", "/b/", "/b/a/", "/c/", "/b/child/", "/b/a/child/", "/c/child/"]);
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn four_cycle_one_move_reverted_with_children() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/child/").unwrap();
    c1.create_at_path("/b/child/").unwrap();
    c1.create_at_path("/c/child/").unwrap();
    c1.create_at_path("/d/child/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c1, "/b/", "/c/").unwrap();
    move_by_path(&c1, "/c/", "/d/").unwrap();
    move_by_path(&c2, "/d/", "/a/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(
        &c2,
        &[
            "/",
            "/d/",
            "/d/c/",
            "/d/c/b/",
            "/d/c/b/a/",
            "/d/child/",
            "/d/c/child/",
            "/d/c/b/child/",
            "/d/c/b/a/child/",
        ],
    );
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn four_cycle_two_moves_reverted_adjacent_with_children() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/child/").unwrap();
    c1.create_at_path("/b/child/").unwrap();
    c1.create_at_path("/c/child/").unwrap();
    c1.create_at_path("/d/child/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c1, "/b/", "/c/").unwrap();
    move_by_path(&c2, "/c/", "/d/").unwrap();
    move_by_path(&c2, "/d/", "/a/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(
        &c2,
        &[
            "/",
            "/c/",
            "/c/b/",
            "/c/b/a/",
            "/d/",
            "/c/child/",
            "/c/b/child/",
            "/c/b/a/child/",
            "/d/child/",
        ],
    );
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn four_cycle_two_moves_reverted_alternating_with_children() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/child/").unwrap();
    c1.create_at_path("/b/child/").unwrap();
    c1.create_at_path("/c/child/").unwrap();
    c1.create_at_path("/d/child/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c2, "/b/", "/c/").unwrap();
    move_by_path(&c1, "/c/", "/d/").unwrap();
    move_by_path(&c2, "/d/", "/a/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(
        &c2,
        &[
            "/",
            "/b/",
            "/b/a/",
            "/d/",
            "/d/c/",
            "/b/child/",
            "/b/a/child/",
            "/d/child/",
            "/d/c/child/",
        ],
    );
    assert_all_document_contents(&c2, &[]);
}

#[test]
fn four_cycle_three_moves_reverted_with_children() {
    let c1 = test_core_with_account();
    c1.create_at_path("/a/child/").unwrap();
    c1.create_at_path("/b/child/").unwrap();
    c1.create_at_path("/c/child/").unwrap();
    c1.create_at_path("/d/child/").unwrap();
    c1.sync(None).unwrap();

    let c2 = another_client(&c1);
    c2.sync(None).unwrap();

    move_by_path(&c1, "/a/", "/b/").unwrap();
    move_by_path(&c2, "/b/", "/c/").unwrap();
    move_by_path(&c2, "/c/", "/d/").unwrap();
    move_by_path(&c2, "/d/", "/a/").unwrap();

    sync_and_assert_stuff(&c1, &c2);
    assert_all_paths(
        &c2,
        &["/", "/b/", "/b/a/", "/c/", "/d/", "/b/child/", "/b/a/child/", "/c/child/", "/d/child/"],
    );
    assert_all_document_contents(&c2, &[]);
}