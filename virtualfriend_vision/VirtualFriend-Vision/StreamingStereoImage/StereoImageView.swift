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
    let width: Int
    let height: Int
    let scale: Float

    @State private var didRender: BoolWrapper = BoolWrapper()
    @State var displayTask: Task<(), Error>?

    let drawableQueue: TextureResource.DrawableQueue
    let context: CIContext

    let stereoImageChannel: AsyncChannel<StereoImage>

    // We add a margin around the displayed image so there aren't wraparound textures displayed on the sides
    let MARGIN: Int = 1

    init(width: Int, height: Int, scale: Float, stereoImageChannel: AsyncChannel<StereoImage>) {
        self.width = width
        self.height = height
        self.scale = scale

        // Two screens, margin on either side = 4 * MARGIN
        self.drawableQueue = try! TextureResource.DrawableQueue(.init(pixelFormat: .bgra8Unorm, width: width * 2 + MARGIN * 4, height: height + MARGIN * 2, usage: [.renderTarget, .shaderRead, .shaderWrite], mipmapsMode: .none))
        self.drawableQueue.allowsNextDrawableTimeout = false

        self.context = CIContext()

        self.stereoImageChannel = stereoImageChannel
    }

    var body: some View {
        RealityView { content in
            if var material = await StereoImageMaterial.shared.material {
                let entity = ModelEntity(mesh: .generatePlane(width: self.scale * Float(self.width) / Float(self.height), height: self.scale))
                content.add(entity)

                // This will appear if it doesn't receive a value from the DrawableQueue quickly enough
                let baseColor = CIImage(color: .black).cropped(to: CGRect(origin: .zero, size: .init(width: self.width * 2 + MARGIN * 4, height: self.height + MARGIN * 2)))
                let image = self.context.createCGImage(baseColor, from: baseColor.extent)!

                do {
                    let texture = try await TextureResource.generate(from: image, options: .init(semantic: .color))
                    texture.replace(withDrawables: self.drawableQueue)

                    try material.setParameter(name: "Image", value: .textureResource(texture))
                } catch {
                    fatalError(error.localizedDescription)
                }

                entity.model?.materials = [material]
            }
            self.onAppear()
        }
        // This constrains the plane to sit directly on top of the window
        // Unsure why this works at 1+, but not at say 0, .1 (which caused zfighting)
        .frame(minDepth: 1, maxDepth: 1.1)
        // TODO: Change to onReceive
        .onAppear {
            self.onAppear()
        }
        .onDisappear {
            self.didRender.value = false
            self.displayTask?.cancel()
            self.displayTask = nil
        }
    }

    /// Require that both RealityView render and onAppear have triggered before we start receiving frames
    func onAppear() {
        if !self.didRender.value {
            self.didRender.value = true

            return
        }

        // Both onAppear and RealityView render has occured. Wait 10ms and subscribe
        self.displayTask = Task {
            try await Task.sleep(for: .milliseconds(10))

            for await image in self.stereoImageChannel {
                if Task.isCancelled {
                    return
                }

                self.step(image)
            }
        }
    }

    func step(_ image: StereoImage) {
        guard let drawable = try? self.drawableQueue.nextDrawable() else {
            // Repeat
            // TODO: This can stack overflow
            self.step(image)
            return
        }

        let left = image.left
        let right = image.right

        // Time to draw
        let width = left.extent.width + CGFloat(MARGIN) * 2
        let height = left.extent.height + CGFloat(MARGIN) * 2

        let leftBounds = CGRect(x: -CGFloat(MARGIN), y: left.extent.minY - CGFloat(MARGIN), width: width, height: height)
        let rightBounds = CGRect(x: -width - CGFloat(MARGIN), y: left.extent.minY - CGFloat(MARGIN), width: width + right.extent.width + CGFloat(MARGIN) * 2, height: height)
        self.context.render(left, to: drawable.texture, commandBuffer: nil, bounds: leftBounds, colorSpace: CGColorSpace(name: CGColorSpace.sRGB)!)
        self.context.render(right, to: drawable.texture, commandBuffer: nil, bounds: rightBounds, colorSpace: CGColorSpace(name: CGColorSpace.sRGB)!)

        drawable.present()
    }
}

private class BoolWrapper {
    var value: Bool = false
}
