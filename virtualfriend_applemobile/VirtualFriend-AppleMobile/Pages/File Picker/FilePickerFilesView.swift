//
//  FilePickerFilesView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 4/27/24.
//

import SwiftUI

private let viewStyles: [FilePickerViewType] = [.list, .grid];

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
            ToolbarItem(placement: .primaryAction) {
                Menu {
                    Picker("View Style", selection: self.$fileViewType) {
                        ForEach(viewStyles, id: \.self) { style in
                            Button(style.title, systemImage: style.image) {

                            }
                            .help(style.title)
                            .tag(style)
                        }
                    }
                } label: {
                    Label(self.fileViewType.title, systemImage: self.fileViewType.image)
                }
            }

            #if !os(visionOS)
            if #available(iOS 26.0, *) {
                ToolbarSpacer(placement: .primaryAction)
            }
            #endif

            ToolbarItem(placement: .primaryAction) {
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

    var title: String {
        switch (self) {
        case .list:
            "List"
        case .grid:
            "Grid"
        }
    }

    var image: String {
        switch (self) {
        case .list:
            "list.bullet"
        case .grid:
            "square.grid.2x2"
        }
    }
}

#Preview {
    NavigationStack {
        FilePickerFilesView(directoryContents: MOCK_FILE_ENTRIES(), onImport: {})
    }
}
