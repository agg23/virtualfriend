//
//  StereoManifestImageView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 4/27/24.
//

import SwiftUI

struct StereoManifestImageView<T: Equatable>: View {
    @LEDBackgroundColor var ledBackgroundColor;
    @LEDColor var ledColor
    @Enable3D var enable3D

    @State var stereoStreamChannel = AsyncImageChannel()
    @State var task: Task<(), Error>?

    let data: T
    let generateImage: (T, VBColor) -> StereoImage
    let onTap: (() -> Void)?
    let integerScaling: Bool?

    init(data: T, generateImage: @escaping (T, VBColor) -> StereoImage, onTap: (() -> Void)? = nil, integerScaling: Bool? = true) {
        self.data = data
        self.generateImage = generateImage
        self.onTap = onTap
        self.integerScaling = integerScaling
    }

    var body: some View {
        StereoImageView(width: 384, height: 224, scale: 0.1, stereoImageChannel: self.stereoStreamChannel, backgroundColor: self.$ledBackgroundColor, onTap: self.onTap, integerScaling: self.integerScaling, force2D: !self.enable3D)
            .onDisappear {
                self.task?.cancel()
            }
            .onChange(of: self.data, initial: true) { _, _ in
                self.generateImageTask()
            }
            .onChange(of: self.ledColor) { _, _ in
                self.generateImageTask()
            }
    }

    func generateImageTask() {
        self.task?.cancel()

        self.task = Task {
            let stereoImage = self.generateImage(self.data, self.ledColor)

            await self.stereoStreamChannel.channel.send(stereoImage)
        }
    }
}

struct StereoManifestFileEntryImageView: View {
    let entry: FileEntryWithManifest
    let onTap: (() -> Void)?
    let integerScaling: Bool?

    init(entry: FileEntryWithManifest, onTap: (() -> Void)? = nil, integerScaling: Bool? = true) {
        self.entry = entry
        self.onTap = onTap
        self.integerScaling = integerScaling
    }

    var body: some View {
        StereoManifestImageView(data: self.entry, generateImage: { data, ledColor in
            FileEntry.image(from: data.manifest ?? FileEntry.getUnknownManifest(), color: ledColor)
        }, onTap: self.onTap, integerScaling: self.integerScaling)
    }
}

#Preview {
    StereoManifestFileEntryImageView(entry: MOCK_FILE_ENTRY_WITH_MANIFEST())
}
