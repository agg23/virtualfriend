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

    @State var drawableQueue: TextureResource.DrawableQueue
    let context: CIContext

    let stereoImageChannel: AsyncImageChannel

    let onTap: (() -> Void)?

    // We add a margin around the displayed image so there aren't wraparound textures displayed on the sides
    let MARGIN: Int = 1

    init(width: Int, height: Int, scale: Float, stereoImageChannel: AsyncImageChannel, onTap: (() -> Void)? = nil) {
        self.width = width
        self.height = height
        self.scale = scale

        self.context = CIContext()

        self.stereoImageChannel = stereoImageChannel

        self.onTap = onTap

        // Two screens, margin on either side = 4 * MARGIN
        self.drawableQueue = try! TextureResource.DrawableQueue(.init(pixelFormat: .bgra8Unorm, width: width * 2 + MARGIN * 4, height: height + MARGIN * 2, usage: [.renderTarget, .shaderRead, .shaderWrite], mipmapsMode: .none))
        self.drawableQueue.allowsNextDrawableTimeout = true
    }

    var body: some View {
        ZStack {
            // Background to prevent flash when loading
            Color.black

            GeometryReader { geometry in
                RealityView { content in
                    let entity = ModelEntity(mesh: .generatePlane(width: self.scale * Float(self.width) / Float(self.height), height: self.scale))

                    // Set up gesture support
                    entity.generateCollisionShapes(recursive: false)
                    entity.components.set(InputTargetComponent())

                    content.add(entity)

                    guard var material = await StereoImageMaterial.shared.material else {
                        return
                    }

                    // This will appear if it doesn't receive a value from the DrawableQueue quickly enough
                    let baseColor = CIImage(color: .black).cropped(to: CGRect(origin: .zero, size: .init(width: self.width * 2 + MARGIN * 4, height: self.height + MARGIN * 2)))
                    let image = self.context.createCGImage(baseColor, from: baseColor.extent)!

                    do {
                        let texture = try await TextureResource.generate(from: image, options: .init(semantic: .color))
                        texture.replace(withDrawables: self.drawableQueue)

                        try material.setParameter(name: "Image", value: .textureResource(texture))
                    } catch is CancellationError {
                        // Do nothing
                    } catch {
                        fatalError(error.localizedDescription)
                    }

                    entity.model?.materials = [material]

                    self.onAppear()
                } update: { content in
                    guard let entity = content.entities.first as? ModelEntity, let model = entity.model else {
                        return
                    }

                    // Update bounds
                    let leftPoint = content.convert(Point3D(simd_float3(0, 0, 0)), from: .local, to: .scene)
                    let rightPoint = content.convert(Point3D(simd_float3(Float(geometry.size.width), Float(geometry.size.height), 1)), from: .local, to: .scene)

                    let diff = rightPoint - leftPoint

                    let leftBound = model.mesh.bounds.min
                    let rightBound = model.mesh.bounds.max

                    let boundDiff = rightBound - leftBound

                    let xScale = abs(diff.x) / abs(boundDiff.x)
                    let yScale = abs(diff.y) / abs(boundDiff.y)

                    entity.transform.scale = [xScale, yScale, 1.0]
                }
                .gesture(self.tap)
            }
            // This constrains the plane to sit directly on top of the window
            // Unsure why this works at 1+, but not at say 0, .1 (which caused zfighting)
            .frame(minDepth: 1.0, maxDepth: 1.1)
        }
        .aspectRatio(CGSize(width: self.width + MARGIN * 2, height: self.height + MARGIN * 2), contentMode: .fit)
        .onChange(of: self.stereoImageChannel, initial: true, { _, _ in
            self.onAppear()
        })
        .onDisappear {
            self.didRender.value = false
            self.displayTask?.cancel()
            self.displayTask = nil
        }
    }

    var tap: some Gesture {
        SpatialTapGesture()
            .targetedToAnyEntity()
            .onEnded { _ in
                self.onTap?()
            }
    }

    /// Require that both RealityView render and onAppear have triggered before we start receiving frames
    func onAppear() {
        if !self.didRender.value {
            self.didRender.value = true

            return
        }

        self.displayTask?.cancel()
        self.displayTask = nil

        // Both onAppear and RealityView render has occured. Wait 10ms and subscribe
        self.displayTask = Task {
            try await Task.sleep(for: .milliseconds(10))

            for await image in self.stereoImageChannel.channel {
                if Task.isCancelled {
                    return
                }

                await self.step(image)
            }
        }
    }

    func step(_ image: StereoImage) async {
        if Task.isCancelled {
            return
        }

//        guard let drawable = try? self.drawableQueue.nextDrawable() else {
//            // Repeat
//            // TODO: This can stack overflow
//            self.step(image)
//            return
//        }

        var drawable: TextureResource.Drawable? = nil
        var stepCount = 0
        while drawable == nil {
            if Task.isCancelled {
                return
            }

            do {
                drawable = try self.drawableQueue.nextDrawable()
            } catch {
                if stepCount > 2000 {
                    print("Aborting draw")
                    return
                }
                
                stepCount += 1
                try? await Task.sleep(nanoseconds: 1000)
            }
        }

        guard let drawable = drawable else {
            fatalError("Unreachable")
        }

        let left = image.left
        let right = image.right

        // Time to draw
        let width = left.extent.width + CGFloat(MARGIN) * 2
        let height = left.extent.height + CGFloat(MARGIN) * 2

        let leftBounds = CGRect(x: -CGFloat(MARGIN), y: left.extent.minY - CGFloat(MARGIN), width: width, height: height)
        let rightBounds = CGRect(x: -width - CGFloat(MARGIN), y: left.extent.minY - CGFloat(MARGIN), width: width + right.extent.width + CGFloat(MARGIN) * 2, height: height)
        await self.context.render(left, to: drawable.texture, commandBuffer: nil, bounds: leftBounds, colorSpace: CGColorSpace(name: CGColorSpace.sRGB)!)
        await self.context.render(right, to: drawable.texture, commandBuffer: nil, bounds: rightBounds, colorSpace: CGColorSpace(name: CGColorSpace.sRGB)!)

        drawable.present()
    }
}

private class BoolWrapper {
    var value: Bool = false
}
