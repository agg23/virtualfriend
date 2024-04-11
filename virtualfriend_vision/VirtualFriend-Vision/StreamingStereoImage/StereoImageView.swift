//
//  StereoImageView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 4/10/24.
//

import SwiftUI
import RealityKit
import Combine

struct StereoImageView: View {
    let width: Int
    let height: Int
    let scale: Float

    @State private var didRender: BoolWrapper = BoolWrapper()
    @State var cancellables: Set<AnyCancellable> = Set()

    let drawableQueue: TextureResource.DrawableQueue

    let context: CIContext
    fileprivate let renderBuffer: RenderBuffer

    // TODO: Change to straight Combine
    @ObservedObject var stereoImage: StreamingStereoImage

    init(width: Int, height: Int, scale: Float, stereoImage: StreamingStereoImage) {
        self.width = width
        self.height = height
        self.scale = scale

        self.drawableQueue = try! TextureResource.DrawableQueue(.init(pixelFormat: .bgra8Unorm, width: width * 2, height: height, usage: [.renderTarget, .shaderRead, .shaderWrite], mipmapsMode: .none))
        self.drawableQueue.allowsNextDrawableTimeout = false

        self.context = CIContext()
        self.renderBuffer = RenderBuffer(queue: self.drawableQueue, context: self.context, streamingImage: stereoImage)

        self.stereoImage = stereoImage
    }

    var body: some View {
        RealityView { content in
            if var material = await StereoImageMaterial.shared.material {
                let entity = ModelEntity(mesh: .generatePlane(width: self.scale * Float(self.width) / Float(self.height), height: self.scale))
                content.add(entity)
                print("Loaded material")

                // This appears to never be used
                let baseColor = CIImage(color: .black).cropped(to: CGRect(origin: .zero, size: .init(width: self.width * 2, height: self.height)))
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
            self.cancellables.forEach { cancellable in
                cancellable.cancel()
            }
        }
    }

    /// Require that both RealityView render and onAppear have triggered before we start receiving frames
    func onAppear() {
        if !self.didRender.value {
            self.didRender.value = true

            return
        }

        // Both onAppear and RealityView render has occured. Wait 10ms and subscribe
        Task {
            try await Task.sleep(for: .milliseconds(10))
            self.renderBuffer.subscribe().store(in: &self.cancellables)
        }
    }
}

private class BoolWrapper {
    var value: Bool = false
}

private class RenderBuffer {
    var bufferedImage = false

    var queue: TextureResource.DrawableQueue

    let streamingImage: StreamingStereoImage

    let dispatchQueue = DispatchQueue(label: "stereoImage", qos: .userInteractive)

    let context: CIContext

    init(queue: TextureResource.DrawableQueue, context: CIContext, streamingImage: StreamingStereoImage) {
        self.queue = queue
        self.context = context

        self.streamingImage = streamingImage
    }

    func subscribe() -> AnyCancellable {
        return self.streamingImage.$image.sink { _ in
            if (!self.bufferedImage) {
                // Request new buffered image
                self.start()
            } else {
                // We've already buffered an image, so this new one will be the one that is displayed
            }
        }
    }

    private func start() {
        self.bufferedImage = true

        self.dispatchQueue.async {
            self.step()
        }
    }

    private func step() {
        guard let drawable = try? self.queue.nextDrawable() else {
            // Repeat
            // TODO: This can stack overflow
            self.step()
            return
        }

        guard let left = self.streamingImage.image.left, let right = self.streamingImage.image.right else {
            // No image, do nothing
            self.bufferedImage = false

            return
        }

        // Time to draw
        self.context.render(left, to: drawable.texture, commandBuffer: nil, bounds: left.extent, colorSpace: CGColorSpace(name: CGColorSpace.sRGB)!)
        self.context.render(right, to: drawable.texture, commandBuffer: nil, bounds: .init(x: -left.extent.width, y: left.extent.minY, width: left.extent.width + right.extent.width, height: right.extent.height), colorSpace: CGColorSpace(name: CGColorSpace.sRGB)!)

        drawable.present()

        self.bufferedImage = false
    }
}
