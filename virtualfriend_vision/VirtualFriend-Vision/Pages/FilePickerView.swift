//
//  FilePickerView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/26/24.
//

import SwiftUI
import RealityKit

let IMAGE_WIDTH = 420.0
let IMAGE_HEIGHT = 224.0

let GRID_SPACING = 40.0

struct FilePickerView: View {
    @AppStorage("romDirectoryBookmark") var romDirectoryBookmark: Data?

    /// Binding to open fileImporter
    @State var selectFolder = false
    @State var directoryContents: [FileEntry] = []

    var body: some View {
        NavigationStack {
            if (directoryContents.isEmpty) {
                VStack {
                    Text("No titles found. Please select a valid titles folder.")
                        .font(.system(size: 24))
                        .multilineTextAlignment(.center)
                        .foregroundStyle(.secondary)
                        .frame(width: 500)

                    Button {
                        selectFolder.toggle()
                    } label: {
                        Text("Choose folder")
                    }
                }
                .padding(40.0)
            } else {
                FilePickerFilesView(directoryContents: self.directoryContents)
            }
        }
        .onAppear {
            self.populateFromBookmark()
        }
        .customFileImporter(self.$selectFolder, onOpen: { url, bookmark in
            self.romDirectoryBookmark = bookmark

            self.buildDirectoryContents(from: url)
        })
    }

    func populateFromBookmark() {
        guard let bookmarkData = self.romDirectoryBookmark else {
            return
        }

        var isStale = false

        let url = try? URL(resolvingBookmarkData: bookmarkData, bookmarkDataIsStale: &isStale)

        guard let url = url, !isStale else {
            print("Could not resolve bookmark")
            return
        }

        self.buildDirectoryContents(from: url)
    }

    func buildDirectoryContents(from url: URL) {
        do {
            guard url.startAccessingSecurityScopedResource() else {
                print("Could not obtain security scope")
                return
            }

            defer { url.stopAccessingSecurityScopedResource() }

            var error: NSError? = nil
            NSFileCoordinator().coordinate(readingItemAt: url, error: &error) { (url) in
                for case let file as URL in FileManager.default.enumerator(at: url, includingPropertiesForKeys: [.nameKey])! {
                    print("Opening", file.startAccessingSecurityScopedResource())
                }
            }

            let urls = try FileManager.default.contentsOfDirectory(at: url, includingPropertiesForKeys: [.isRegularFileKey])

            let filteredUrls = urls.filter { url in
                url.pathExtension == "vb"
            }.sorted { a, b in
                a.lastPathComponent < b.lastPathComponent
            }

            self.directoryContents = filteredUrls.map { url in
                let _ = url.startAccessingSecurityScopedResource()
                let hash = hashOfFile(atUrl: url)
                url.stopAccessingSecurityScopedResource()

                return FileEntry(url: url, hash: hash)
            }
        } catch {
            // Directory not found
            print(error)
        }
    }
}

#Preview {
    FilePickerView()
}
