//
//  StereoImageVisionView.swift
//  VirtualFriend-AppleMobile
//
//  Created by Adam Gastineau on 6/3/24.
//

import SwiftUI
import RealityKit
import AsyncAlgorithms
import CoreImage
import Metal
import UIKit

#if os(visionOS)
struct StereoImageVisionView: View {
    @State private var textureStore = StereoTextureStore()

    @Binding var backgroundColor: CGColor

    let width: Int
    let height: Int
    let scale: Float

    /// Our own size, observed rather than read from a `GeometryReader`, so a scroll doesn't rebuild
    /// this view (and the `RealityView` under it) on every layout pass
    @State private var size: CGSize = .zero

    let stereoImageChannel: AsyncImageChannel

    let onTap: (() -> Void)?

    let force2D: Bool

    // We add a margin around the displayed image so there aren't wraparound textures displayed on the sides
    let MARGIN: Int = 1

    init(width: Int, height: Int, scale: Float, stereoImageChannel: AsyncImageChannel, backgroundColor: Binding<CGColor>, onTap: (() -> Void)? = nil, force2D: Bool = false) {
        self.width = width
        self.height = height
        self.scale = scale

        self._backgroundColor = backgroundColor
        self.stereoImageChannel = stereoImageChannel

        self.onTap = onTap

        self.force2D = force2D
    }

    var body: some View {
        RealityView { content in
            // Purposefully synchronous
            let entity = ModelEntity(mesh: .generatePlane(width: self.scale * Float(self.width) / Float(self.height), height: self.scale))

            // Set up gesture support
            entity.generateCollisionShapes(recursive: false)
            entity.components.set(InputTargetComponent())

            // Default material until the stereo material is ready
            entity.model?.materials = [PlaceholderMaterial.material(for: self.backgroundColor)]

            content.add(entity)

            self.textureStore.attach(entity)

            if self.size != .zero {
                _ = self.textureStore.didSizeChange(to: self.size)

                self.scale(entity, in: content)
            }
        } update: { content in
            guard let entity = content.entities.first as? ModelEntity else {
                return
            }

            // Since `update` is called constantly on SwiftUI invalidation, let's make sure there's new geometry to consume before doing real work
            guard self.size != .zero, self.textureStore.didSizeChange(to: self.size) else {
                return
            }

            self.scale(entity, in: content)
        } placeholder: {
            // Replaces the default spinner
            Color(cgColor: self.backgroundColor)
        }
        // Fires only when size changes
        .onGeometryChange(for: CGSize.self) { proxy in
            proxy.size
        } action: { newSize in
            self.size = newSize
        }
        // Stream frames to the texture
        .task(id: FrameStreamId(channel: self.stereoImageChannel, force2D: self.force2D)) {
            guard let texture = self.texture() else {
                return
            }

            // On startup, we don't know what might be in the texture buffer. Make sure to clear it
            texture.clearIfEmpty(to: self.backgroundColor)

            // This may take non-zero time, so run it outside of main render
            await self.textureStore.attachMaterial(for: texture)

            // Always render the latest frame only; no backpressure
            for await image in self.stereoImageChannel.channel.buffer(policy: .bufferingLatest(1)) {
                if Task.isCancelled {
                    return
                }

                texture.write(image, background: self.backgroundColor, force2D: self.force2D)
            }
        }
    }

    private func scale(_ entity: ModelEntity, in content: RealityViewContent) {
        guard let model = entity.model else {
            return
        }

        let leftPoint = content.convert(Point3D(simd_float3(0, 0, 0)), from: .local, to: .scene)
        let rightPoint = content.convert(Point3D(simd_float3(Float(self.size.width), Float(self.size.height), 1)), from: .local, to: .scene)

        let diff = rightPoint - leftPoint

        let leftBound = model.mesh.bounds.min
        let rightBound = model.mesh.bounds.max

        let boundDiff = rightBound - leftBound

        let xScale = abs(diff.x) / abs(boundDiff.x)
        let yScale = abs(diff.y) / abs(boundDiff.y)

        entity.transform.scale = [xScale, yScale, 1.0]
    }

    private func texture() -> StereoTexture? {
        self.textureStore.texture(width: self.width, height: self.height, margin: self.MARGIN)
    }

    private struct FrameStreamId: Equatable {
        let channel: AsyncImageChannel
        let force2D: Bool
    }
}

@MainActor
private enum PlaceholderMaterial {
    private static var cached: (color: CGColor, material: UnlitMaterial)?

    static func material(for color: CGColor) -> UnlitMaterial {
        if let cached = PlaceholderMaterial.cached, cached.color == color {
            return cached.material
        }

        let material = UnlitMaterial(color: UIColor(cgColor: color))
        PlaceholderMaterial.cached = (color, material)

        return material
    }
}

/// Lazily creates and owns the texture for a single view
@MainActor
private final class StereoTextureStore {
    private var stored: StereoTexture?
    private var failed = false

    private var entity: ModelEntity?
    private var material: ShaderGraphMaterial?

    private var lastSize: CGSize?

    func didSizeChange(to size: CGSize) -> Bool {
        guard self.lastSize != size else {
            return false
        }

        self.lastSize = size

        return true
    }

    func attach(_ entity: ModelEntity) {
        self.entity = entity

        self.applyMaterial()
    }

    func attachMaterial(for texture: StereoTexture) async {
        guard self.material == nil else {
            return
        }

        guard var material = await StereoImageMaterial.shared.material else {
            return
        }

        do {
            try material.setParameter(name: "Image", value: .textureResource(texture.resource))
        } catch {
            print("Could not bind stereo texture to material: \(error)")

            return
        }

        self.material = material

        self.applyMaterial()
    }

    private func applyMaterial() {
        guard let entity = self.entity, let material = self.material else {
            return
        }

        entity.model?.materials = [material]
    }

    func texture(width: Int, height: Int, margin: Int) -> StereoTexture? {
        if let stored = self.stored {
            return stored
        }

        guard !self.failed else {
            return nil
        }

        do {
            let texture = try StereoTexture(width: width, height: height, margin: margin)
            self.stored = texture

            return texture
        } catch {
            print("Could not create stereo texture: \(error)")

            // Don't retry every frame
            self.failed = true

            return nil
        }
    }
}

@MainActor
private final class StereoTexture {
    // Command queue and context is shared across all views in the app
    private static let commandQueue: MTLCommandQueue? = MTLCreateSystemDefaultDevice()?.makeCommandQueue()

    private static let context: CIContext? = {
        guard let commandQueue = StereoTexture.commandQueue else {
            return nil
        }

        return CIContext(mtlCommandQueue: commandQueue, options: [.cacheIntermediates: false])
    }()

    private static let PIXEL_FORMAT: MTLPixelFormat = .bgra8Unorm
    private static let COLOR_SPACE = CGColorSpace(name: CGColorSpace.displayP3)!

    let width: Int
    let height: Int
    let margin: Int

    let resource: TextureResource

    /// Whether anything has been drawn into the texture
    private var hasContent = false

    private let texture: LowLevelTexture

    /// Two screens, margin on either side
    var totalWidth: Int {
        self.width * 2 + self.margin * 4
    }

    var totalHeight: Int {
        self.height + self.margin * 2
    }

    init(width: Int, height: Int, margin: Int) throws {
        self.width = width
        self.height = height
        self.margin = margin

        var descriptor = LowLevelTexture.Descriptor()
        descriptor.textureType = .type2D
        descriptor.pixelFormat = StereoTexture.PIXEL_FORMAT
        descriptor.width = width * 2 + margin * 4
        descriptor.height = height + margin * 2
        descriptor.textureUsage = [.renderTarget, .shaderRead, .shaderWrite]

        self.texture = try LowLevelTexture(descriptor: descriptor)
        self.resource = try TextureResource(from: self.texture)
    }

    /// Fill the whole texture with a solid color if no content has been drawn yet
    func clearIfEmpty(to color: CGColor) {
        guard !self.hasContent else {
            return
        }

        self.hasContent = true

        self.encode { context, destination in
            try context.startTask(toRender: StereoTexture.image(for: color), to: destination)
        }
    }

    func write(_ image: StereoImage, background: CGColor, force2D: Bool) {
        self.hasContent = true

        let left = image.left
        let right = force2D ? image.left : image.right

        let margin = CGFloat(self.margin)
        let placedLeft = left.transformed(by: .init(translationX: margin, y: margin))
        let placedRight = right.transformed(by: .init(translationX: CGFloat(self.width) + margin * 3, y: margin))

        let frame = placedLeft
            .composited(over: placedRight)
            .composited(over: StereoTexture.image(for: background))

        self.encode { context, destination in
            try context.startTask(toRender: frame, to: destination)
        }
    }

    private func encode(_ body: (CIContext, CIRenderDestination) throws -> Void) {
        guard let commandQueue = StereoTexture.commandQueue,
              let context = StereoTexture.context,
              let commandBuffer = commandQueue.makeCommandBuffer() else {
            return
        }

        let texture = self.texture.replace(using: commandBuffer)

        let destination = CIRenderDestination(width: self.totalWidth, height: self.totalHeight, pixelFormat: StereoTexture.PIXEL_FORMAT, commandBuffer: commandBuffer, mtlTextureProvider: {
            texture
        })
        destination.colorSpace = StereoTexture.COLOR_SPACE

        do {
            try body(context, destination)
        } catch {
            print("Could not render stereo frame: \(error)")
        }

        commandBuffer.commit()
    }

    private static func image(for color: CGColor) -> CIImage {
        CIImage(color: CIColor(cgColor: color))
    }
}
#endif
