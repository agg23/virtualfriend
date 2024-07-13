//
//  FilePickerFilesView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 4/27/24.
//

import SwiftUI

struct FilePickerFilesView: View {
    @AppStorage("fileViewType") fileprivate var fileViewType: FilePickerViewType = .list

    let files: [FileEntryWithManifest]
    let onImport: () -> Void

    init(directoryContents: [FileEntry], onImport: @escaping () -> Void) {
        self.files = directoryContents.map { entry in
            entry.withManifest
        }
        self.onImport = onImport
    }

    var body: some View {
        Group {
            switch self.fileViewType {
            case .list:
                #if os(iOS)
                FilePickerListiOSView(files: self.files)
                #else
                FilePickerListView(files: self.files)
                #endif
            case .grid:
                FilePickerGrid(files: self.files)
            }
        }
        .navigationTitle("Library")
        .toolbar {
            ToolbarItem(placement: .navigation) {
                HStack {
                    Picker("View Style", selection: self.$fileViewType) {
                        Button("List", systemImage: "list.bullet") {

                        }
                        .help("List")
                        .tag(FilePickerViewType.list)

                        Button("Grid", systemImage: "square.grid.2x2") {

                        }
                        .help("Grid")
                        .tag(FilePickerViewType.grid)
                    }
                    .pickerStyle(.segmented)

                    Spacer()
                }
            }

            ToolbarItem(placement: .topBarTrailing) {
                Button("Import Titles", systemImage: "plus") {
                    self.onImport()
                }
            }
        }
    }
}

private enum FilePickerViewType: String {
    case list
    case grid
}

#Preview {
    NavigationStack {
        FilePickerFilesView(directoryContents: MOCK_FILE_ENTRIES(), onImport: {})
    }
}
