//
//  FileImporter.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 5/5/24.
//

import Foundation

enum DirectoryContentsState {
    case loading
    case data([FileEntry])
}

@Observable class FileImporter {
    let titlesDirectory: URL
    var directoryContents: DirectoryContentsState = .loading

    init() {
        var documents = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
        documents.append(component: "Titles")
        self.titlesDirectory = documents
    }

    func fetchKnownTitles() -> [String: URL] {
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

        return knownTitles
    }

    func rescanTitles() {
        self.buildEntries(self.fetchKnownTitles())
    }

    func delete(at url: URL) {
        try? FileManager.default.removeItem(at: url)
        self.rescanTitles()
    }

    func importFiles(from url: URL) {
        var files: [URL] = []

        // Start by rescanning existing titles
        var knownTitles = self.fetchKnownTitles()

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

            guard let hash = hashOfFile(atUrl: file), knownTitles[hash] == nil else {
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

            knownTitles[hash] = destinationUrl
        }

        url.stopAccessingSecurityScopedResource()

        self.buildEntries(knownTitles)
    }

    private func buildEntries(_ knownTitles: [String: URL]) {
        self.directoryContents = .data(knownTitles.filter { (_, url) in
            url.pathExtension == "vb"
        }.sorted { a, b in
            a.value.lastPathComponent < b.value.lastPathComponent
        }.map { (hash, url) in
            FileEntry(url: url, hash: hash)
        })
    }
}
