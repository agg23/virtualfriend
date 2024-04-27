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

    init(directoryContents: [FileEntry]) {
        self.files = directoryContents.map { entry in
            entry.withManifest
        }
    }

    var body: some View {
        Group {
            switch self.fileViewType {
            case .list:
                FilePickerListView(files: self.files)
            case .grid:
                FilePickerGrid(files: self.files)
            }
        }
        .animation(.default, value: self.fileViewType)
        .toolbar {
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
        }

//        NavigationSplitView(columnVisibility: splitViewVisibility, sidebar: {
//            Group {
//                switch self.fileViewType {
//                case .list:
//                    Text("list")
//                case .grid:
//                    FilePickerGrid(directoryContents: self.$directoryContents)
//                }
//            }
//            Text("list")
//        }, detail: {
//            Text("Foo")
//                .toolbar {
//                    Text("")
//                    Picker("View Style", selection: self.$fileViewType) {
//                        Button("List", systemImage: "list.bullet") {
//
//                        }
//                        .tag(FilePickerViewType.list)
//
//                        Button("Grid", systemImage: "square.grid.2x2") {
//
//                        }
//                        .tag(FilePickerViewType.grid)
//                    }
//                    .pickerStyle(.segmented)
//                }
//        })
    }
}

private enum FilePickerViewType: String {
    case list
    case grid
}

#Preview {
    NavigationStack {
        FilePickerFilesView(directoryContents: MOCK_FILE_ENTRIES())
    }
}
