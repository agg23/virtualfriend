//
//  FilePickerListView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 4/27/24.
//

import SwiftUI

struct FilePickerListView: View {
    @Environment(MainRouter.self) private var router

    @State var selectedFile: FileEntryWithManifest?

    let files: [FileEntryWithManifest]

    var body: some View {
        Grid(horizontalSpacing: 0) {
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
    }

    @ViewBuilder
    var list: some View {
        List(selection: self.$selectedFile) {
            ForEach(self.files, id: \.entry.id) { entry in
                HStack {
                    Text(self.title(from: entry))
                        .font(.title)
                        .lineLimit(1)
                    if let metadata = entry.manifest?.metadata {
                        Text(metadata.publisher.toString() + " " + metadata.year.toString())
                            .lineLimit(1)
                    } else {
                        // Placeholder
                        // TODO: There should be something better that can be done here
                        Text(" ")
                    }
                }
                .tag(entry)
            }
        }
        .onChange(of: self.files, initial: true) { _, newValue in
            if self.selectedFile == nil {
                self.selectedFile = newValue.first
            }
        }
    }

    @ViewBuilder
    var details: some View {
        ZStack {
            Color.black.opacity(0.4)
                .ignoresSafeArea()

            if let selectedFile = self.selectedFile {
                VStack {
                    StereoManifestImageView(entry: selectedFile)

                    Text(self.title(from: selectedFile))
                        .font(.largeTitle)
                        .lineLimit(1)

                    // TODO: Replace this
                    if let metadata = selectedFile.manifest?.metadata {
                        Text(metadata.publisher.toString() + " " + metadata.year.toString())
                            .lineLimit(1)
                    } else {
                        // Placeholder
                        // TODO: There should be something better that can be done here
                        Text(" ")
                    }

                    Button("Play", systemImage: "play.fill") {
                        self.router.currentRoute = .emulator(url: selectedFile.entry.url)
                    }
                    .padding()

                    Spacer()

                    if selectedFile.manifest == nil {
                        Text(selectedFile.entry.url.lastPathComponent)
                            .font(.footnote)
                            .padding()
                    }
                }
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
