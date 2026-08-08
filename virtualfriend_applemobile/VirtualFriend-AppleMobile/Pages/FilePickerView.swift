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

    var body: some View {
        NavigationStack {
            switch self.fileImporter.directoryContents {
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
        .environment(self.fileImporter)
        .onAppear {
            // Since this view tree gets destroyed when opening the emulator, this onAppear is not a duplicate with the scenePhase onChange
            self.fileImporter.rescanTitles()
        }
        .onChange(of: self.scenePhase, { prevValue, nextValue in
            if nextValue == .active && prevValue != .active {
                // We're becoming active after not previously being. Rebuild directories
                self.fileImporter.rescanTitles()
            }
        })
        .customFileImporter(self.$selectFolder, onOpen: { url, _ in
            Task {
                self.fileImporter.importFiles(from: url)
            }
        })
    }
}

#Preview {
    FilePickerView()
}
