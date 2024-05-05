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
    @State var fileImporter = FileImporter()

    /// Binding to open fileImporter
    @State var selectFolder = false
    @State var directoryContents: [FileEntry] = []

    var body: some View {
        NavigationStack {
            FilePickerFilesView(directoryContents: self.directoryContents) {
                self.selectFolder.toggle()
            }
        }
        .onAppear {
            self.fileImporter.rescanTitles()

            self.buildEntries()
        }
        .customFileImporter(self.$selectFolder, onOpen: { url, _ in
            Task {
                self.fileImporter.importFiles(from: url)

                self.buildEntries()
            }
        })
    }

    func buildEntries() {
        self.directoryContents = self.fileImporter.knownTitles.filter { (_, url) in
            url.pathExtension == "vb"
        }.sorted { a, b in
            a.value.lastPathComponent < b.value.lastPathComponent
        }.map { (hash, url) in
            FileEntry(url: url, hash: hash)
        }
    }
}

#Preview {
    FilePickerView()
}
