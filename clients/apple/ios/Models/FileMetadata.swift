//
//  FileMetadata.swift
//  ios_client
//
//  Created by Raayan Pillai on 4/11/20.
//  Copyright © 2020 Lockbook. All rights reserved.
//

import Foundation

struct FileMetadata: Codable, Identifiable {
    var id: String
    var name: String
    var path: String
    var updatedAt: Int
    var version: Int
    var status: Status
}

enum Status: String, Codable {
    case New
    case Local
    case Remote
    case Synced
}