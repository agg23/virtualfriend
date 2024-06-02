//
//  FileImporter.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 5/5/24.
//

import Foundation

struct FileImporter {
    let titlesDirectory: URL
    var knownTitles: [String: URL] = [:]

    init() {
        var documents = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
        documents.append(component: "Titles")
        self.titlesDirectory = documents
    }

    mutating func rescanTitles() {
        var knownTitles: [String: URL] = [:]

        do {
            let titleContents = try FileManager.default.contentsOfDirectory(at: self.titlesDirectory, includingPropertiesForKeys: [.isRegularFileKey])

            for titleURL in titleContents.filter({ $0.pathExtension == "vb" }) {
                if let hash = hashOfFile(atUrl: titleURL) {
                    knownTitles[hash] = titleURL
                }
            }
        } catch {
            print("Could not load title directory contents \(error)")
        }

        self.knownTitles = knownTitles
    }

    mutating func importFiles(from url: URL) {
        var files: [URL] = []

        // Start by rescanning existing titles
        self.rescanTitles()

        let _ = url.startAccessingSecurityScopedResource()

        if !url.isDirectory {
            // User selected a single file
            files = [url]
        } else if let enumerator = FileManager.default.enumerator(at: url, includingPropertiesForKeys: [.isRegularFileKey], options: [.skipsHiddenFiles, .skipsPackageDescendants]) {
            for case let fileUrl as URL in enumerator {
                do {
                    let attributes = try fileUrl.resourceValues(forKeys: [.isRegularFileKey])

                    if attributes.isRegularFile! && fileUrl.pathExtension == "vb" {
                        files.append(fileUrl)
                    }
                } catch {
                    print("Couldn't resolve resource values \(error)")
                }
            }
        }

        for file in files {
            // Attempt to open it, but ignore the result
            let _ = file.startAccessingSecurityScopedResource()

            defer { file.stopAccessingSecurityScopedResource() }

            guard let hash = hashOfFile(atUrl: file), self.knownTitles[hash] == nil else {
                print("Importing \(file) failed due to duplicate hash")

                continue
            }

            var filename = file.lastPathComponent

            if let manifest = FileEntry(url: file, hash: hash).manifest, let title = manifest.metadata?.title {
                // We have manifest, use as file name
                filename = "\(title.toString()).vb"
            }

            var destinationUrl = self.titlesDirectory
            destinationUrl.append(component: filename)

            print("Importing \(file) to \(destinationUrl)")

            do {
                try FileManager.default.copyItem(at: file, to: destinationUrl)
            } catch {
                print("Could not copy file \(file) to \(destinationUrl): \(error)")
            }

            self.knownTitles[hash] = destinationUrl
        }

        url.stopAccessingSecurityScopedResource()
    }
}
