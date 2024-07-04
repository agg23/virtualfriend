//
//  FilePickerEntry.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/27/24.
//

import SwiftUI
import AsyncAlgorithms

struct FilePickerGridItemView: View {
    @Environment(MainRouter.self) private var router

    let CORNER_RADIUS = 20.0

    @LEDBackgroundColor var ledBackgroundColor;

    let entry: FileEntryWithManifest

    let stereoStreamChannel: AsyncImageChannel

    init(entry: FileEntryWithManifest) {
        self.entry = entry

        let channel = AsyncImageChannel()
        self.stereoStreamChannel = channel
    }

    var body: some View {
        let metadata = self.entry.manifest?.metadata

        #if os(visionOS)
        // Extra 16 to allow button press to keep 3D view hidden
        let videoAspectRatio = 384.0/(224.0 + 16.0)
        #else
        let videoAspectRatio = 384.0/224.0
        #endif

        let videoView = StereoManifestImageView(entry: self.entry, onTap: {
            self.router.currentRoute = .emulator(entry: self.entry)
        }, integerScaling: false)

        let button = Button {
            self.router.currentRoute = .emulator(entry: self.entry)
        } label: {
            VStack {
                #if os(visionOS)
                self.ledBackgroundColor
                    .aspectRatio(videoAspectRatio, contentMode: .fit)
                    .ignoresSafeArea(edges: .horizontal)
                #else
                videoView
                #endif

                Text(self.entry.title)
                    #if os(visionOS)
                    .font(.title)
                    #else
                    .font(.title3)
                    #endif
                    .lineLimit(1)
                Group {
                    if let metadata = metadata {
                        Text(metadata.publisher.toString() + " " + metadata.year.toString())
                            .lineLimit(1)
                            .foregroundStyle(.secondary)
                    } else {
                        // Placeholder
                        // TODO: There should be something better that can be done here
                        Text(" ")
                    }
                }
                .padding(.bottom, 8)
            }
            #if os(visionOS)
            .background(.tertiary)
//                    //                .clipShape(.rect(cornerRadius: 20.0))
            .contentShape(.contextMenuPreview, RoundedRectangle(cornerRadius: CORNER_RADIUS))
            .cornerRadius(CORNER_RADIUS)
            #endif
        }
        // Custom button style as we can't make the black above span the entire width of the button without it
        .buttonStyle(.plain)
        .buttonBorderShape(.roundedRectangle(radius: CORNER_RADIUS))

        ZStack {
            #if os(visionOS)
            button
            VStack {
                videoView
                    .padding()

                Spacer()
            }
            #else
            button
            #endif
        }
    }
}

#Preview {
    FilePickerGridItemView(entry: MOCK_FILE_ENTRY_WITH_MANIFEST())
}
