//
//  FilePickerGrid.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 4/27/24.
//

import SwiftUI

struct FilePickerGrid: View {
    static let GRID_SPACING = 40.0

    let files: [FileEntryWithManifest]

    var body: some View {
        ScrollView {
            LazyVGrid(columns: [
                GridItem(spacing: FilePickerGrid.GRID_SPACING),
                GridItem(spacing: FilePickerGrid.GRID_SPACING),
                GridItem(spacing: FilePickerGrid.GRID_SPACING),
            ], spacing: FilePickerGrid.GRID_SPACING) {
                ForEach(self.files, id: \.entry.id) { entry in
                    FilePickerGridItemView(entry: entry)
                }
            }
            .padding([.horizontal, .bottom], 24)
        }
    }
}

#Preview {
    FilePickerGrid(files: MOCK_FILE_ENTRIES_WITH_MANIFESTS())
}
