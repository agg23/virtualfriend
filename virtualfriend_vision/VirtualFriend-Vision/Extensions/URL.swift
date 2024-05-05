//
//  URL.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 5/5/24.
//

import Foundation

extension URL {
    var isDirectory: Bool {
        (try? self.resourceValues(forKeys: [.isDirectoryKey]))?.isDirectory ?? false
    }
}
