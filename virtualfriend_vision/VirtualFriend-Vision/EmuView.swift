//
//  EmuView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/21/24.
//

import SwiftUI
import RealityKit
import VBStereoRenderRealityKit
import Combine

let PIXEL_WIDTH = 384
let PIXEL_HEIGHT = 224
let PIXEL_COUNT = PIXEL_WIDTH * PIXEL_HEIGHT
let PIXEL_BYTE_COUNT = PIXEL_COUNT * 4

struct EmuView: View {
    let queue: DispatchQueue
    let virtualFriend: VirtualFriend!

    @State var image: CGImage

    let context = CIContext()

    let drawableQueue = try! TextureResource.DrawableQueue(.init(pixelFormat: .bgra8Unorm, width: 384, height: 224, usage: [.renderTarget, .shaderRead, .shaderWrite], mipmapsMode: .none))

    var leftImage = UIImage(named: "Left")!
    var rightImage = UIImage(named: "Right")!

    @State private var cancellables = Set<AnyCancellable>()

    init() {
        self.queue = DispatchQueue(label: "emu", qos: .userInteractive)

        var data = [UInt8](repeating: 0, count: PIXEL_BYTE_COUNT)

        for i in 0..<PIXEL_BYTE_COUNT {
            data[i] = 0
        }

        let colorspace = CGColorSpaceCreateDeviceRGB()
        let rgbData = CFDataCreate(nil, data, PIXEL_BYTE_COUNT)!
        let provider = CGDataProvider(data: rgbData)!
        let image = CGImage(width: PIXEL_WIDTH,
                       height: PIXEL_HEIGHT,
                       bitsPerComponent: 8,
                       bitsPerPixel: 8 * 3,
                       bytesPerRow: PIXEL_WIDTH * 3,
                       space: colorspace,
                       bitmapInfo: CGBitmapInfo(rawValue: 0),
                       provider: provider,
                       decode: nil,
                       shouldInterpolate: true,
                       intent: CGColorRenderingIntent.defaultIntent)!
        self.image = image

        let url = Bundle.main.url(forResource: "Mario's Tennis (Japan, USA)", withExtension: "vb")

        guard let url = url else {
            assertionFailure("Could not find embedded ROM")
            self.virtualFriend = nil
            return
        }

        self.virtualFriend = VirtualFriend(url.path(percentEncoded: false))
    }

    var body: some View {
        RealityView { content in
            // Add the initial RealityKit content
            if let scene = try? await Entity(named: "Scene", in: vBStereoRenderRealityKitBundle) {
                content.add(scene)

                let cube = scene.findEntity(named: "HoldingCube") as? ModelEntity

                guard var model = cube?.model, var material = model.materials.first as? ShaderGraphMaterial else {
                    fatalError("Cannot load material")
                }

                let baseColor = CIImage(color: .red).cropped(to: CGRect(origin: .zero, size: .init(width: 384, height: 224)))
                let image = context.createCGImage(baseColor, from: baseColor.extent)!

                do {
                    let texture = try await TextureResource.generate(from: image, options: .init(semantic: .raw))

                    texture.replace(withDrawables: self.drawableQueue)

                    try material.setParameter(name: "Left_Image", value: .textureResource(texture))
                } catch {
                    fatalError(error.localizedDescription)
                }

                model.materials = [material]
                cube?.model = model
            }
        }
        .onAppear(perform: {
            self.queue.async {
                while (true) {
//                    let frame = self.virtualFriend.run_frame()
//                    let ciImage = rustVecToCIImage(frame.left)
                    
                    let ciImage = CIImage(image: self.leftImage)!
                    // TODO: This should be flipped by Metal, not the CPU
                    let transformedImage = ciImage.transformed(by: .init(scaleX: 1, y: -1))
                    self.image = context.createCGImage(transformedImage, from: transformedImage.extent)!

                    do {
                        let drawable = try drawableQueue.nextDrawable()

                        context.render(transformedImage, to: drawable.texture, commandBuffer: .none, bounds: transformedImage.extent, colorSpace: CGColorSpace(name: CGColorSpace.displayP3)!)

                        drawable.present()
                    } catch {
                        print(error.localizedDescription)
                    }
                }
            }
        })
        Image(self.image, scale: 1.0, label: Text("Hi"))
    }
}

func rustVecToCIImage(_ vec: RustVec<UInt8>) -> CIImage {
    var bytes = [UInt8](repeating: 0, count: PIXEL_BYTE_COUNT)

    for i in 0..<PIXEL_COUNT {
        let value = vec[i]

        bytes[i * 4] = value
        bytes[i * 4 + 1] = 0
        bytes[i * 4 + 2] = 0
        // Alpha
        bytes[i * 4 + 3] = 255
    }

    let bitmapData = Data(bytes)

    return CIImage(bitmapData: bitmapData, bytesPerRow: PIXEL_WIDTH * 4, size: .init(width: PIXEL_WIDTH, height: PIXEL_HEIGHT), format: .RGBA8, colorSpace: .none)
}

#Preview {
    EmuView()
}
