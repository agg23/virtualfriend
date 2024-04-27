//
//  FilePickerGrid.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 4/27/24.
//

import SwiftUI

struct FilePickerGrid: View {
    static let IMAGE_WIDTH = 420.0
    static let IMAGE_HEIGHT = 224.0

    static let GRID_SPACING = 40.0

    let files: [FileEntryWithManifest]

    private let columns = [
        GridItem(.fixed(IMAGE_WIDTH), spacing: GRID_SPACING, alignment: nil),
        GridItem(.fixed(IMAGE_WIDTH), spacing: GRID_SPACING, alignment: nil),
        GridItem(.fixed(IMAGE_WIDTH), spacing: GRID_SPACING, alignment: nil)
    ]

    var body: some View {
        ScrollView {
            LazyVGrid(columns: self.columns, spacing: FilePickerGrid.GRID_SPACING) {
                ForEach(self.files, id: \.entry.id) { entry in
                    FilePickerEntryView(entry: entry, imageWidth: FilePickerGrid.IMAGE_WIDTH, imageHeight: FilePickerGrid.IMAGE_HEIGHT)
                }
            }
        }
    }
}

#Preview {
    FilePickerGrid(files: MOCK_FILE_ENTRIES_WITH_MANIFESTS())
}
