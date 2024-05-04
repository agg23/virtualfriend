//
//  StereoManifestImageView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 4/27/24.
//

import SwiftUI

struct StereoManifestImageView: View {
    @LEDBackgroundColor var ledBackgroundColor;
    @LEDForegroundColor var ledForegroundColor;

    @State var stereoStreamChannel = AsyncImageChannel()
    @State var task: Task<(), Error>?

    let entry: FileEntryWithManifest
    let onTap: (() -> Void)?

    init(entry: FileEntryWithManifest, onTap: (() -> Void)? = nil) {
        self.entry = entry
        self.onTap = onTap
    }

    var body: some View {
        StereoImageView(width: 384, height: 224, scale: 0.1, stereoImageChannel: self.stereoStreamChannel, backgroundColor: self.$ledBackgroundColor, onTap: self.onTap)
            .onDisappear {
                self.task?.cancel()
            }
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
        self.task?.cancel()

        self.task = Task {
            let stereoImage = FileEntry.image(from: self.entry.manifest ?? FileEntry.getUnknownManifest(), foregroundColor: self.ledForegroundColor.rawCGColor, backgroundColor: self.ledBackgroundColor.rawCGColor)

            await self.stereoStreamChannel.channel.send(stereoImage)
        }
    }
}

#Preview {
    StereoManifestImageView(entry: MOCK_FILE_ENTRY_WITH_MANIFEST())
}
