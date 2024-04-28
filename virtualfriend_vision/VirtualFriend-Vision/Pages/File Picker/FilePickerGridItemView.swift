//
//  FilePickerEntry.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/27/24.
//

import SwiftUI
import AsyncAlgorithms

struct FilePickerGridItemView: View {
    @Environment(\.openWindow) var openWindow

    let CORNER_RADIUS = 20.0

    let entry: FileEntryWithManifest

    let stereoStreamChannel: AsyncImageChannel

    init(entry: FileEntryWithManifest) {
        self.entry = entry

        guard let manifest = self.entry.manifest else {
            let manifest = FileEntry.getUnknownManifest()
            let stereoImage = FileEntry.image(from: manifest)

            let channel = AsyncImageChannel()
            Task {
                await channel.channel.send(stereoImage)
            }
            self.stereoStreamChannel = channel

            return
        }

        let stereoImage = FileEntry.image(from: manifest)

        let channel = AsyncImageChannel()
        Task {
            await channel.channel.send(stereoImage)
        }
        self.stereoStreamChannel = channel
    }

    var body: some View {
        let metadata = self.entry.manifest?.metadata

        ZStack {
            Button {
                openWindow(id: "emu", value: self.entry.entry.url)
            } label: {
                VStack {
                    Color.black
                        // Extra 16 to allow button press to keep 3D view hidden
                        .aspectRatio(384.0/(224.0 + 16.0), contentMode: .fit)
                        .ignoresSafeArea(edges: .horizontal)

                    Text(metadata?.title.toString() ?? self.entry.entry.url.deletingPathExtension().lastPathComponent)
                        .font(.title)
                        .lineLimit(1)
                    Group {
                        if let metadata = metadata {
                            Text(metadata.publisher.toString() + " " + metadata.year.toString())
                                .lineLimit(1)
                        } else {
                            // Placeholder
                            // TODO: There should be something better that can be done here
                            Text(" ")
                        }
                    }
                    .padding(.bottom, 8)
                }
                .background(.tertiary)
//                    //                .clipShape(.rect(cornerRadius: 20.0))
                .contentShape(.contextMenuPreview, RoundedRectangle(cornerRadius: CORNER_RADIUS))
                .cornerRadius(CORNER_RADIUS)
            }
            // Custom button style as we can't make the black above span the entire width of the button without it
            .buttonStyle(.plain)
            .buttonBorderShape(.roundedRectangle(radius: CORNER_RADIUS))

            VStack {
                StereoImageView(width: 384, height: 224, scale: 0.1, stereoImageChannel: self.stereoStreamChannel) {
                    openWindow(id: "emu", value: self.entry.entry.url)
                }
                .padding()

                Spacer()
            }
        }
    }
}

#Preview {
    FilePickerGridItemView(entry: MOCK_FILE_ENTRY_WITH_MANIFEST())
}
