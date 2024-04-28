//
//  StereoManifestImageView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 4/27/24.
//

import SwiftUI
import AsyncAlgorithms

struct StereoManifestImageView: View {
    @State var stereoStreamChannel = AsyncImageChannel()

    let entry: FileEntryWithManifest

    var body: some View {
        StereoImageView(width: 384, height: 224, scale: 0.1, stereoImageChannel: self.stereoStreamChannel)
            .task(id: self.entry) {
                Task {
                    let stereoImage = FileEntry.image(from: self.entry.manifest ?? FileEntry.getUnknownManifest())

                    await self.stereoStreamChannel.channel.send(stereoImage)
                }
            }
    }
}

#Preview {
    StereoManifestImageView(entry: MOCK_FILE_ENTRY_WITH_MANIFEST())
}
