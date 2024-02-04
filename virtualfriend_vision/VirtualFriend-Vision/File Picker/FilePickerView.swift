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
//    @AppStorage("romDirectory") var romDirectory: String?
    @State var romDirectory: URL?

    @State var selectFolder = false

    private var directoryContents: Binding<[(URL, String?)]> {
        Binding {
//            guard let romDirectory = self.romDirectory, let url = URL(string: romDirectory) else {
//                return []
//            }
            guard let url = self.romDirectory else {
                return []
            }

            do {
                print(url.startAccessingSecurityScopedResource())

                var error: NSError? = nil
                NSFileCoordinator().coordinate(readingItemAt: url, error: &error) { (url) in
                    for case let file as URL in FileManager.default.enumerator(at: url, includingPropertiesForKeys: [.nameKey])! {
                        print("Opening", file.startAccessingSecurityScopedResource())
                    }
                }

                let urls = try FileManager.default.contentsOfDirectory(at: url, includingPropertiesForKeys: [.isRegularFileKey])

//                url.stopAccessingSecurityScopedResource()

                let filteredUrls = urls.filter { url in
                    url.pathExtension == "vb"
                }.sorted { a, b in
                    a.lastPathComponent < b.lastPathComponent
                }

                return filteredUrls.map { url in
                    let _ = url.startAccessingSecurityScopedResource()
                    let hash = hashOfFile(atUrl: url)
                    url.stopAccessingSecurityScopedResource()

                    return (url, hash)
                }
            } catch {
                // Directory not found
                print(error)
                return []
            }
        } set: { _, _ in

        }
    }

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
                LazyVGrid(columns: self.columns, spacing: GRID_SPACING) {
                    // Make sure we always have 9 items and insert placeholders
                    ForEach(0..<max(9, self.directoryContents.count), id: \.self) { index in
                        if index < self.directoryContents.count {
                            let file = self.directoryContents[index]

                            FilePickerEntry(fileUrl: file.0, hash: file.1, imageWidth: IMAGE_WIDTH, imageHeight: IMAGE_HEIGHT)
                                .frame(height: IMAGE_HEIGHT + 70.0)
                        } else {
                            Color(.clear)
                                .frame(height: IMAGE_HEIGHT + 70.0)
                                .hidden()
                        }
                    }
                }
            }
        }
        .padding(40.0)
        .fileImporter(isPresented: $selectFolder, allowedContentTypes: [.folder]) { result in
            switch result {
            case .success(let url):
//                self.romDirectory = url.absoluteString
                self.romDirectory = url
            case .failure(let failure):
                print("\(failure)")
            }
        }
    }
}

#Preview {
    FilePickerView()
}
