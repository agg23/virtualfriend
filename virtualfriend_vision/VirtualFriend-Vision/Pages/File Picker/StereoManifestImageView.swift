//
//  StereoManifestImageView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 4/27/24.
//

import SwiftUI
import AsyncAlgorithms

struct StereoManifestImageView: View {
    @LEDBackgroundColor var ledBackgroundColor;
    @LEDForegroundColor var ledForegroundColor;

    @State var stereoStreamChannel = AsyncImageChannel()

    let entry: FileEntryWithManifest

    var body: some View {
        StereoImageView(width: 384, height: 224, scale: 0.1, stereoImageChannel: self.stereoStreamChannel)
            .onChange(of: self.entry, initial: true) { _, _ in
                self.generateImage()
            }
            .onChange(of: self.ledBackgroundColor) { _, _ in
                self.generateImage()
            }
            .onChange(of: self.ledForegroundColor) { _, _ in
                self.generateImage()
            }
    }

    func generateImage() {
        Task {
            let stereoImage = FileEntry.image(from: self.entry.manifest ?? FileEntry.getUnknownManifest(), foregroundColor: self.ledForegroundColor.rawCGColor, backgroundColor: self.ledBackgroundColor.rawCGColor)

            print("Sending image")
            await self.stereoStreamChannel.channel.send(stereoImage)
            print("Sent image")
        }
    }
}

#Preview {
    StereoManifestImageView(entry: MOCK_FILE_ENTRY_WITH_MANIFEST())
}
