//
//  LockbookApi.swift
//  ios_client
//
//  Created by Raayan Pillai on 4/11/20.
//  Copyright © 2020 Lockbook. All rights reserved.
//

import Foundation

protocol LockbookApi {
    func getAccount() -> Optional<String>
    func createAccount(username: String) -> Bool
    func importAccount(accountString: String) -> Bool
    func updateMetadata() -> [FileMetadata]
    func createFile(name: String) -> Optional<FileMetadata>
    func getFile(id: String) -> Optional<DecryptedValue>
    func updateFile(id: String, content: String) -> Bool
    func purgeLocal() -> Bool
}

struct CoreApi: LockbookApi {
    let documentsDirectory: String
    
    private func isDbPresent() -> Bool {
        if (is_db_present(documentsDirectory) == 1) {
            return true
        }
        return false
    }
    
    func getAccount() -> Optional<String> {
        if (isDbPresent()) {
            let result = get_account(documentsDirectory)
            let resultString = String(cString: result!)
            release_pointer(UnsafeMutablePointer(mutating: result))
            return Optional.some(resultString)
        }
        return Optional.none
    }

    func createAccount(username: String) -> Bool {
        let result = create_account(documentsDirectory, username)
        if (result == 1) {
            return true
        }
        return false
    }
    
    func importAccount(accountString: String) -> Bool {
        let result = import_account(documentsDirectory, accountString)
        if (result == 1) {
            return true
        }
        return false
    }
    
    func updateMetadata() -> [FileMetadata] {
        if (isDbPresent()) {
            let result = sync_files(documentsDirectory)
            let resultString = String(cString: result!)
            // We need to release the pointer once we have the result string
            release_pointer(UnsafeMutablePointer(mutating: result))
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            
            if let resultMetas: [FileMetadata] = deserialize(jsonStr: resultString) {
                return resultMetas
            } else {
                return [FileMetadata].init()
            }
        }
        return []
    }
    
    func createFile(name: String) -> Optional<FileMetadata> {
        let result = create_file(documentsDirectory, name, "")
        let resultString = String(cString: result!)
        release_pointer(UnsafeMutablePointer(mutating: result))
        
        let resultMeta: Optional<FileMetadata> = deserialize(jsonStr: resultString)
        return resultMeta
    }
    
    func getFile(id: String) -> Optional<DecryptedValue> {
        let result = get_file(documentsDirectory, id)
        let resultString = String(cString: result!)
        release_pointer(UnsafeMutablePointer(mutating: result))
        
        let resultFile: Optional<DecryptedValue> = deserialize(jsonStr: resultString)
        return resultFile
    }
    
    func updateFile(id: String, content: String) -> Bool {
        let result = update_file(documentsDirectory, id, content)
        if (result == 1) {
            return true
        }
        return false
    }
    
    func purgeLocal() -> Bool {
        if(purge_files(documentsDirectory) == 1) {
            return true
        }
        return false
    }
}


struct FakeApi: LockbookApi {
    var fakeUsername: String = "FakeApi"
    var fakeMetadatas: [FileMetadata] = [
        FileMetadata(id: "aaaa", name: "first_file.md", path: "/", updatedAt: 0, version: 0, status: .Synced),
        FileMetadata(id: "bbbb", name: "another_file.md", path: "/", updatedAt: 1000, version: 1000, status: .Synced),
        FileMetadata(id: "cccc", name: "third_file.md", path: "/", updatedAt: 1500, version: 1500, status: .Local),
    ]
    
    func getAccount() -> Optional<String> {
        Optional.some(fakeUsername)
    }
    
    func createAccount(username: String) -> Bool {
        false
    }
    
    func importAccount(accountString: String) -> Bool {
        false
    }
    
    func updateMetadata() -> [FileMetadata] {
        var rander = SystemRandomNumberGenerator()
        return fakeMetadatas.shuffled(using: &rander)
    }
    
    func createFile(name: String) -> Optional<FileMetadata> {
        let now = Date().timeIntervalSince1970

        return Optional.some(FileMetadata(id: "new", name: name, path: "", updatedAt: Int(now), version: Int(now), status: .Local))
    }
    
    func getFile(id: String) -> Optional<DecryptedValue> {
        Optional.none
    }
    
    func updateFile(id: String, content: String) -> Bool {
        false
    }
    
    func purgeLocal() -> Bool {
        false
    }
}