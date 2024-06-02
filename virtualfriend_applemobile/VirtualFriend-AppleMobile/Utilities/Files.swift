//
//  Files.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/28/24.
//

import Foundation

extension FileManager {
    func getFilesWithExtension(at url: URL, fileExtension: String) -> [URL] {
        let files: [URL]

        do {
            files = try FileManager.default.contentsOfDirectory(at: url, includingPropertiesForKeys: [.isRegularFileKey])
        } catch {
            return []
        }

        return files.filter { url in
            url.pathExtension == fileExtension
        }
    }
}
