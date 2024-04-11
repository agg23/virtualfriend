//
//  StreamingStereoImageView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/24/24.
//

import SwiftUI
import RealityKit
import Combine

import VBStereoRenderRealityKit

struct StreamingStereoImageView: View {
    let width: Int
    let height: Int

    let zPosition: Float
    let scale: Float

    // TODO: Remove
    let dispatchQueue = DispatchQueue(label: "stereoImage", qos: .userInteractive)
    @State var cancellables: Set<AnyCancellable> = Set()

    let leftDrawableQueue: TextureResource.DrawableQueue
    let rightDrawableQueue: TextureResource.DrawableQueue

    fileprivate let renderBuffer: RenderBuffer


    let context = CIContext()

    @State var isFirstDraw = true
    @ObservedObject var stereoImage: StreamingStereoImage

    init(width: Int, height: Int, stereoImage: StreamingStereoImage, zPosition: Float = 0.0, scale: Float = 1.0) {
        self.width = width
        self.height = height

        self.zPosition = zPosition
        self.scale = scale

        self.leftDrawableQueue = try! TextureResource.DrawableQueue(.init(pixelFormat: .bgra8Unorm, width: width, height: height, usage: [.renderTarget, .shaderRead, .shaderWrite], mipmapsMode: .none))
        self.rightDrawableQueue = try! TextureResource.DrawableQueue(.init(pixelFormat: .bgra8Unorm, width: width, height: height, usage: [.renderTarget, .shaderRead, .shaderWrite], mipmapsMode: .none))

        self.renderBuffer = RenderBuffer(leftQueue: self.leftDrawableQueue, rightQueue: self.rightDrawableQueue, streamingImage: stereoImage)

        self.leftDrawableQueue.allowsNextDrawableTimeout = false
        self.rightDrawableQueue.allowsNextDrawableTimeout = false

        self.stereoImage = stereoImage
    }

    var body: some View {
        RealityView { content in
            if let scene = await StereoImageScene.shared.scene {
                content.add(scene)

                let cube = scene.findEntity(named: "VBStereoRenderRealityKit")

                // Float display slightly above window
                cube?.position = [0.0, 0.0, self.zPosition]
                cube?.scale = [self.scale * Float(self.width) / Float(self.height), self.scale, 0.0]

                let mesh = cube?.children.first(where: { $0.name == "HoldingCube" }) as? ModelEntity

                guard var model = mesh?.model, var material = model.materials.first as? ShaderGraphMaterial else {
                    fatalError("Cannot load material")
                }

                // This appears to never be used
                let baseColor = CIImage(color: .black).cropped(to: CGRect(origin: .zero, size: .init(width: self.width, height: self.height)))
                let image = context.createCGImage(baseColor, from: baseColor.extent)!

                do {
                    let leftTexture = try await TextureResource.generate(from: image, options: .init(semantic: .color))
                    leftTexture.replace(withDrawables: self.leftDrawableQueue)

                    try material.setParameter(name: "Left_Image", value: .textureResource(leftTexture))

                    let rightTexture = try await TextureResource.generate(from: image, options: .init(semantic: .color))
                    rightTexture.replace(withDrawables: self.rightDrawableQueue)

                    try material.setParameter(name: "Right_Image", value: .textureResource(rightTexture))
                } catch {
                    fatalError(error.localizedDescription)
                }

                model.materials = [material]
                mesh?.model = model
            }

        }
        .onAppear {
//            self.stereoImage.$image.sink { image in
//                // TODO: Is this only run on initial update?
////                let retryCount = isFirstDraw ? 50 : 0
//                presentImage(image.left, queue: self.leftDrawableQueue, isLeft: true)
//                presentImage(image.right, queue: self.rightDrawableQueue, isLeft: false)
////
////                isFirstDraw = false
////                if (!self.renderBuffer.bufferedImage) {
////                    // Buffer new image
////                    
////                } else {
////                    // Do nothing
////                }
//            }
//            .store(in: &self.cancellables)
            self.renderBuffer.subscribe().store(in: &self.cancellables)
        }
    }

    // Keeping this around for the moment, as RenderBuffer may be finicky
//    func presentImage(_ image: CIImage?, queue: TextureResource.DrawableQueue, isLeft: Bool) {
//        self.dispatchQueue.async {
//            guard let image = image else {
//                return
//            }
//
//            do {
//                let drawable = try queue.nextDrawable()
//
//                context.render(image, to: drawable.texture, commandBuffer: .none, bounds: image.extent, colorSpace: CGColorSpace(name: CGColorSpace.sRGB)!)
//
//                drawable.present()
//            } catch {
//                print("Could not update \(isLeft ? "left" : "right") drawable: \(error)")
////                if (retryCount > 0) {
//                    presentImage(image, queue: queue, isLeft: isLeft)
////                }
//            }
//        }
//    }
}

private class RenderBuffer {
    var bufferedImage = false

    var leftQueue: TextureResource.DrawableQueue
    var rightQueue: TextureResource.DrawableQueue

    var leftDrawable: TextureResource.Drawable?
    var rightDrawable: TextureResource.Drawable?

    let streamingImage: StreamingStereoImage

    let dispatchQueue = DispatchQueue(label: "stereoImage", qos: .userInteractive)

    let context = CIContext()

    init(leftQueue: TextureResource.DrawableQueue, rightQueue: TextureResource.DrawableQueue, streamingImage: StreamingStereoImage) {
        self.leftQueue = leftQueue
        self.rightQueue = rightQueue

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
        if (self.leftDrawable == nil) {
            do {
                self.leftDrawable = try self.leftQueue.nextDrawable()
            } catch {
                // Try next step
                // print("\(error)")
            }
        }

        if (self.rightDrawable == nil) {
            do {
                self.rightDrawable = try self.rightQueue.nextDrawable()
            } catch {
                // Try next step
                // print("\(error)")
            }
        }

        guard let leftDrawable = self.leftDrawable, let rightDrawable = self.rightDrawable else {
            // Repeat
            self.dispatchQueue.async {
                self.step()
            }
            return
        }

        guard let left = self.streamingImage.image.left, let right = self.streamingImage.image.right else {
            // No image, do nothing
            self.bufferedImage = false
            self.leftDrawable = nil
            self.rightDrawable = nil

            return
        }

        // Time to draw
        self.context.render(left, to: leftDrawable.texture, commandBuffer: .none, bounds: left.extent, colorSpace: CGColorSpace(name: CGColorSpace.sRGB)!)
        self.context.render(right, to: rightDrawable.texture, commandBuffer: .none, bounds: right.extent, colorSpace: CGColorSpace(name: CGColorSpace.sRGB)!)

        leftDrawable.present()
        rightDrawable.present()

        self.bufferedImage = false
        self.leftDrawable = nil
        self.rightDrawable = nil
    }
}

#Preview {
    StreamingStereoImageView(width: 384, height: 224, stereoImage: StreamingStereoImage(image: StereoImage(left: nil, right: nil)))
}
