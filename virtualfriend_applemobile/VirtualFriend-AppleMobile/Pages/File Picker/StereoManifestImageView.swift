//
//  StereoManifestImageView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 4/27/24.
//

import SwiftUI

struct StereoManifestImageView: View {
    @LEDBackgroundColor var ledBackgroundColor;
    @LEDColor var ledColor

    @State var stereoStreamChannel = AsyncImageChannel()
    @State var task: Task<(), Error>?

    let entry: FileEntryWithManifest
    let onTap: (() -> Void)?
    let integerScaling: Bool?

    init(entry: FileEntryWithManifest, onTap: (() -> Void)? = nil, integerScaling: Bool? = true) {
        self.entry = entry
        self.onTap = onTap
        self.integerScaling = integerScaling
    }

    var body: some View {
        StereoImageView(width: 384, height: 224, scale: 0.1, stereoImageChannel: self.stereoStreamChannel, backgroundColor: self.$ledBackgroundColor, onTap: self.onTap, integerScaling: self.integerScaling)
            .onDisappear {
                self.task?.cancel()
            }
            .onChange(of: self.entry, initial: true) { _, _ in
                self.generateImage()
            }
            .onChange(of: self.ledColor) { _, _ in
                self.generateImage()
            }
    }

    func generateImage() {
        self.task?.cancel()

        self.task = Task {
            let stereoImage = FileEntry.image(from: self.entry.manifest ?? FileEntry.getUnknownManifest(), color: self.ledColor)

            await self.stereoStreamChannel.channel.send(stereoImage)
        }
    }
}

#Preview {
    StereoManifestImageView(entry: MOCK_FILE_ENTRY_WITH_MANIFEST())
}
