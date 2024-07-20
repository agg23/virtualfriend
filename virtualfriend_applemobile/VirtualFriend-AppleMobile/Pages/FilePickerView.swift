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
    @Environment(\.scenePhase) private var scenePhase

    @State private var fileImporter = FileImporter()

    /// Binding to open fileImporter
    @State private var selectFolder = false
    @State private var directoryContents: DirectoryContentsState = .loading

    var body: some View {
        NavigationStack {
            switch self.directoryContents {
            case .loading:
                ProgressView()
            case .data(let directoryContents):
                if (directoryContents.isEmpty) {
                    VStack {
                        Text("No titles found. Please select folders or files to import.")
                            .font(.system(size: 24))
                            .multilineTextAlignment(.center)
                            .foregroundStyle(.secondary)
                            #if os(visionOS)
                            .frame(width: 500)
                            #endif

                        Button {
                            self.selectFolder.toggle()
                        } label: {
                            Text("Import Titles")
                        }
                        .padding(.top, 16)
                    }
                    .padding(40.0)
                } else {
                    FilePickerFilesView(directoryContents: directoryContents) {
                        self.selectFolder.toggle()
                    }
                }
            }
        }
        .onAppear {
            self.fileImporter.rescanTitles()

            self.buildEntries()
        }
        .onChange(of: self.scenePhase, { prevValue, nextValue in
            if nextValue == .active && prevValue != .active {
                // We're becoming active after not previously being. Rebuild directories
                self.fileImporter.rescanTitles()

                self.buildEntries()
            }
        })
        .customFileImporter(self.$selectFolder, onOpen: { url, _ in
            Task {
                self.fileImporter.importFiles(from: url)

                self.buildEntries()
            }
        })
    }

    func buildEntries() {
        self.directoryContents = .data(self.fileImporter.knownTitles.filter { (_, url) in
            url.pathExtension == "vb"
        }.sorted { a, b in
            a.value.lastPathComponent < b.value.lastPathComponent
        }.map { (hash, url) in
            FileEntry(url: url, hash: hash)
        })
    }

    enum DirectoryContentsState {
        case loading
        case data([FileEntry])
    }
}

#Preview {
    FilePickerView()
}
