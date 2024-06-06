//
//  FilePickerGrid.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 4/27/24.
//

import SwiftUI

struct FilePickerGrid: View {
    #if os(visionOS)
    static let GRID_SPACING = 40.0
    #else
    static let GRID_SPACING = 20.0
    #endif

    let files: [FileEntryWithManifest]

    #if os(visionOS)
    let columns = [
        GridItem(spacing: FilePickerGrid.GRID_SPACING),
        GridItem(spacing: FilePickerGrid.GRID_SPACING),
        GridItem(spacing: FilePickerGrid.GRID_SPACING),
    ]
    #else
    let columns = [GridItem(.adaptive(minimum: 140), spacing: FilePickerGrid.GRID_SPACING)]
    #endif

    var body: some View {
        ScrollView {
            LazyVGrid(columns: self.columns, spacing: FilePickerGrid.GRID_SPACING) {
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
