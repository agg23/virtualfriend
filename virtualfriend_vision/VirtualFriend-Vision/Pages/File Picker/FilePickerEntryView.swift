//
//  FilePickerEntry.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/27/24.
//

import SwiftUI
import AsyncAlgorithms

struct FilePickerEntryView: View {
    @Environment(\.openWindow) var openWindow

    let imageWidth: CGFloat
    let imageHeight: CGFloat

    let entry: FileEntryWithManifest

    let stereoStreamChannel: AsyncImageChannel

    init(entry: FileEntryWithManifest, imageWidth: CGFloat, imageHeight: CGFloat) {
        self.entry = entry

        self.imageWidth = imageWidth
        self.imageHeight = imageHeight

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
                    // Placeholder of the size of the StreamingStereoImageView
                    Color(.clear)
                        .frame(width: self.imageWidth, height: self.imageHeight)
                    Text(metadata?.title.toString() ?? self.entry.entry.url.deletingPathExtension().lastPathComponent)
                        .font(.title)
                    if let metadata = metadata {
                        Text(metadata.publisher.toString() + " " + metadata.year.toString())
                    } else {
                        // Placeholder
                        // TODO: There should be something better that can be done here
                        Text(" ")
                    }
                }
            }
            .buttonBorderShape(.roundedRectangle(radius: 20.0))

            VStack {
//                ZStack {
//                    Color(.green)
//                        .frame(width: self.imageWidth, height: self.imageHeight)
//                StreamingStereoImageView(width: 384, height: 224, stereoImage: stereoImage, scale: 0.7)
//                    .frame(width: self.imageWidth, height: self.imageHeight)
                StereoImageView(width: 384, height: 224, scale: 0.1, stereoImageChannel: self.stereoStreamChannel)
//                }
                Spacer()
            }
        }
    }
}

#Preview {
    FilePickerEntryView(entry: MOCK_FILE_ENTRY_WITH_MANIFEST(), imageWidth: 480, imageHeight: 300)
}
