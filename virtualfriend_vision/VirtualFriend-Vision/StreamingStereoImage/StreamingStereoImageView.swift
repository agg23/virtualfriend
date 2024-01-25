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

    let dispatchQueue: DispatchQueue
    @State var cancellables: Set<AnyCancellable>

    let leftDrawableQueue: TextureResource.DrawableQueue
    let rightDrawableQueue: TextureResource.DrawableQueue

    let context = CIContext()

    @ObservedObject var stereoImage: StreamingStereoImage

    init(width: Int, height: Int, stereoImage: StreamingStereoImage) {
        self.width = width
        self.height = height

        self.dispatchQueue = DispatchQueue(label: "stereoImage", qos: .userInteractive)
        self.cancellables = Set()

        self.leftDrawableQueue = try! TextureResource.DrawableQueue(.init(pixelFormat: .bgra8Unorm, width: width, height: height, usage: [.renderTarget, .shaderRead, .shaderWrite], mipmapsMode: .none))
        self.rightDrawableQueue = try! TextureResource.DrawableQueue(.init(pixelFormat: .bgra8Unorm, width: width, height: height, usage: [.renderTarget, .shaderRead, .shaderWrite], mipmapsMode: .none))

        self.stereoImage = stereoImage
    }

    var body: some View {
        RealityView { content in
            if let scene = try? await Entity(named: "Scene", in: vBStereoRenderRealityKitBundle) {
                content.add(scene)

                let cube = scene.findEntity(named: "VBStereoRenderRealityKit")
                let mesh = cube?.children.first(where: { $0.name == "HoldingCube" }) as? ModelEntity

                guard var model = mesh?.model, var material = model.materials.first as? ShaderGraphMaterial else {
                    fatalError("Cannot load material")
                }

                let baseColor = CIImage(color: .black).cropped(to: CGRect(origin: .zero, size: .init(width: 384, height: 224)))
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
            self.stereoImage.$image.sink { image in
                presentImage(image.left, queue: self.leftDrawableQueue)
                presentImage(image.right, queue: self.rightDrawableQueue)
            }
            .store(in: &self.cancellables)
        }
    }

    func presentImage(_ image: CIImage?, queue: TextureResource.DrawableQueue) {
        self.dispatchQueue.async {
            guard let image = image else {
                return
            }

            do {
                let drawable = try queue.nextDrawable()

                context.render(image, to: drawable.texture, commandBuffer: .none, bounds: image.extent, colorSpace: CGColorSpace(name: CGColorSpace.sRGB)!)

                drawable.present()
            } catch {
                print("Could not update left drawable: \(error)")
            }
        }
    }
}

#Preview {
    StreamingStereoImageView(width: 384, height: 224, stereoImage: StreamingStereoImage(image: StereoImage(left: nil, right: nil)))
}
