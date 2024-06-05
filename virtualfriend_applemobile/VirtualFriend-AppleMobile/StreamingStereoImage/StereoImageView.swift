//
//  StereoImageView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 4/10/24.
//

import SwiftUI
import RealityKit
import AsyncAlgorithms

struct StereoImageView: View {
    @Binding private var backgroundColor: CGColor

    let width: Int
    let height: Int
    let scale: Float

    let stereoImageChannel: AsyncImageChannel

    let onTap: (() -> Void)?
    let integerScaling: Bool?

    // We add a margin around the displayed image so there aren't wraparound textures displayed on the sides
    let MARGIN: Int = 1

    init(width: Int, height: Int, scale: Float, stereoImageChannel: AsyncImageChannel, backgroundColor: Binding<Color>? = nil, onTap: (() -> Void)? = nil, integerScaling: Bool? = true) {
        self.width = width
        self.height = height
        self.scale = scale

        if let backgroundColor = backgroundColor {
            self._backgroundColor = backgroundColor.rawCGColor
        } else {
            self._backgroundColor = Binding {
                CGColor(red: 0.0, green: 0.0, blue: 0.0, alpha: 1.0)
            } set: { _ in }
        }
        self.stereoImageChannel = stereoImageChannel

        self.onTap = onTap
        self.integerScaling = integerScaling
    }

    var body: some View {
        ZStack {
            // Background to prevent flash when loading
            Color(cgColor: self.backgroundColor)

            GeometryReader { geometry in
                #if os(visionOS)
                let contentView = StereoImageVisionView(width: self.width, height: self.height, scale: self.scale, geometry: geometry, stereoImageChannel: self.stereoImageChannel, backgroundColor: self.$backgroundColor)
                #else
                let contentView = Metal2DView(stereoImageChannel: self.stereoImageChannel, size: CGSize(width: self.width, height: self.height), integerScaling: self.integerScaling ?? true)
                #endif

                if self.onTap != nil {
                    contentView
                        .gesture(self.tap)
                } else {
                    contentView
                }
            }
            #if os(visionOS)
            // This constrains the plane to sit directly on top of the window
            // Unsure why this works at 1+, but not at say 0, .1 (which caused zfighting)
            // Higher depth to allow tapping on the view in EmuView
            .frame(minDepth: 4.0, maxDepth: 4.1)
            #endif
        }
        .aspectRatio(CGSize(width: self.width + MARGIN * 2, height: self.height + MARGIN * 2), contentMode: .fit)
    }

    var tap: some Gesture {
        SpatialTapGesture()
            #if os(visionOS)
            .targetedToAnyEntity()
            #endif
            .onEnded { _ in
                self.onTap?()
            }
    }
}
