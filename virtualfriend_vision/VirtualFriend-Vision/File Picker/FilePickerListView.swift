//
//  FilePickerListView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 4/27/24.
//

import SwiftUI

struct FilePickerListView: View {
    @Environment(\.openWindow) var openWindow

    @State var selectedFile: FileEntryWithManifest?

    let files: [FileEntryWithManifest]

    var body: some View {
        Grid {
            GridRow {
                ForEach(0..<3) { _ in
                    EmptyView()
                }
            }
            GridRow {
                self.list
                    .gridCellColumns(2)
                self.details
            }
        }
//        List(self.files, id: \.entry.id) { entry in
//            let metadata = entry.manifest?.metadata
//
//            Button {
//                openWindow(id: "emu", value: entry.entry.url)
//            } label: {
//                HStack {
//                    Text(metadata?.title.toString() ?? entry.entry.url.deletingPathExtension().lastPathComponent)
//                        .font(.title)
//                    if let metadata = metadata {
//                        Text(metadata.publisher.toString() + " " + metadata.year.toString())
//                    } else {
//                        // Placeholder
//                        // TODO: There should be something better that can be done here
//                        Text(" ")
//                    }
//                }
//            }
//        }
    }

    @ViewBuilder
    var list: some View {
        List(selection: self.$selectedFile) {
            ForEach(self.files, id: \.entry.id) { entry in
                let metadata = entry.manifest?.metadata

                HStack {
                    Text(self.title(from: entry))
                        .font(.title)
                    if let metadata = metadata {
                        Text(metadata.publisher.toString() + " " + metadata.year.toString())
                    } else {
                        // Placeholder
                        // TODO: There should be something better that can be done here
                        Text(" ")
                    }
                }
                .tag(entry)
            }
        }
    }

    @ViewBuilder
    var details: some View {
        ZStack {
            Color.black.opacity(0.4)
                .ignoresSafeArea()

            if let selectedFile = self.selectedFile {
                StereoManifestImageView(entry: selectedFile)
                    .id(selectedFile)

                Text(self.title(from: selectedFile))
            }
        }
    }

    func title(from entry: FileEntryWithManifest) -> String {
        entry.manifest?.metadata?.title.toString() ?? entry.entry.url.deletingPathExtension().lastPathComponent
    }
}

#Preview {
    FilePickerListView(files: MOCK_FILE_ENTRIES_WITH_MANIFESTS())
}
