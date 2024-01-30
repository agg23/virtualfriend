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
    @AppStorage("romDirectory") var romDirectory: String?

    @State var selectFolder = false

    private var directoryContents: Binding<[(URL, String?)]> {
        Binding {
            guard let romDirectory = self.romDirectory, let url = URL(string: romDirectory) else {
                return []
            }

            do {
                let urls = try FileManager.default.contentsOfDirectory(at: url, includingPropertiesForKeys: [.isRegularFileKey])

                let filteredUrls = urls.filter { url in
                    url.pathExtension == "vb"
                }.sorted { a, b in
                    a.lastPathComponent < b.lastPathComponent
                }

                return filteredUrls.map { url in
                    let hash = hashOfFile(atUrl: url)

                    return (url, hash)
                }
            } catch {
                // Directory not found
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
                    ForEach(self.directoryContents, id: \.0) { file in
                        FilePickerEntry(fileUrl: file.0, hash: file.1, imageWidth: IMAGE_WIDTH, imageHeight: IMAGE_HEIGHT)
                    }
                }
            }
        }
        .fileImporter(isPresented: $selectFolder, allowedContentTypes: [.folder]) { result in
            switch result {
            case .success(let url):
                self.romDirectory = url.absoluteString
            case .failure(let failure):
                print("\(failure)")
            }
        }
    }
}

#Preview {
    FilePickerView()
}
