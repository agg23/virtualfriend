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
    @State var directoryContents: [(URL, String?)] = []

    private let columns = [
        GridItem(.fixed(IMAGE_WIDTH), spacing: GRID_SPACING, alignment: nil),
        GridItem(.fixed(IMAGE_WIDTH), spacing: GRID_SPACING, alignment: nil),
        GridItem(.fixed(IMAGE_WIDTH), spacing: GRID_SPACING, alignment: nil)
    ]

    var body: some View {
        VStack {
            if (directoryContents.isEmpty) {
                Text("No games found. Please select a valid ROMs folder.")
                    .font(.system(size: 24))
                    .multilineTextAlignment(.center)
                    .foregroundStyle(.secondary)
                    .frame(width: 500)

                Button {
                    selectFolder.toggle()
                } label: {
                    Text("Choose folder")
                }
            } else {
                ScrollView {
                    LazyVGrid(columns: self.columns, spacing: GRID_SPACING) {
                        // Make sure we always have 9 items and insert placeholders
                        ForEach(self.$directoryContents, id: \.0) { urlAndHash in
                            FilePickerEntry(fileUrl: urlAndHash.0, hash: urlAndHash.1, imageWidth: IMAGE_WIDTH, imageHeight: IMAGE_HEIGHT)
                        }
                    }
                }
            }
        }
        .padding(40.0)
        .onAppear {
            self.populateFromBookmark()
        }
        .fileImporter(isPresented: $selectFolder, allowedContentTypes: [.folder]) { result in
            switch result {
            case .success(let url):
                self.buildDirectoryContents(from: url)
                print("Selected \(url)")
            case .failure(let failure):
                print("Failed to open folder: \(failure)")
            }
        }
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

            // Update bookmark
            let bookmarkData = try? url.bookmarkData(options: .minimalBookmark)
            self.romDirectoryBookmark = bookmarkData

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

                return (url, hash)
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
