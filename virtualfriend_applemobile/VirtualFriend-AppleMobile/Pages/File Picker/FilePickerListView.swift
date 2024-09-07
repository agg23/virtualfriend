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
                    Text(entry.title)
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
        .onAppear {
            guard let selectedUrl = self.router.selectedFile else {
                return
            }

            // Try to find selected file in all files
            let selectedFile = self.files.first { file in
                file.entry.url == selectedUrl
            }

            guard let selectedFile = selectedFile else {
                return
            }

            self.selectedFile = selectedFile
        }
        .onChange(of: self.files, initial: true) { _, newValue in
            if self.selectedFile == nil {
                self.selectedFile = newValue.first
            }
        }
        .onChange(of: self.selectedFile) { _, newValue in
            guard let selectedFile = newValue else {
                return
            }

            self.router.selectedFile = selectedFile.entry.url
        }
    }

    @ViewBuilder
    var details: some View {
        ZStack {
            Color.black.opacity(0.4)
                .ignoresSafeArea()

            if let selectedFile = self.selectedFile {
                VStack {
                    StereoManifestFileEntryImageView(entry: selectedFile)

                    Text(selectedFile.title)
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
                        self.router.currentRoute = .emulator(entry: selectedFile)
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
}

#Preview {
    FilePickerListView(files: MOCK_FILE_ENTRIES_WITH_MANIFESTS())
}
