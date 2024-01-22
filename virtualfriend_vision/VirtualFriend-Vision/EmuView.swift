//
//  EmuView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/21/24.
//

import SwiftUI
import RealityKit
import VBStereoRenderRealityKit

let PIXEL_WIDTH = 384
let PIXEL_HEIGHT = 224
let PIXEL_COUNT = PIXEL_WIDTH * PIXEL_HEIGHT
let PIXEL_BYTE_COUNT = PIXEL_COUNT * 3

struct EmuView: View {
    let queue: DispatchQueue
    let virtualFriend: VirtualFriend!

    @State var image: CGImage
    @State var activeImage: CGImage

    @State var toggle: Bool

    var leftImage = UIImage(named: "Left")!

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
        self.activeImage = image

        self.toggle = false

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
            }
        } update: { content in
            // Update the RealityKit content when SwiftUI state changes
            if let scene = content.entities.first {
                let cube = scene.findEntity(named: "HoldingCube") as? ModelEntity
                var shader = cube?.model?.materials.first as? ShaderGraphMaterial

                let image = self.toggle ? self.image : self.leftImage.cgImage!

                do {
                    print("Drawing image")
                    let texture = try TextureResource.generate(from: image, withName: "leftTexture", options: TextureResource.CreateOptions.init(semantic: .raw))

                    print(shader?.parameterNames)

                    try shader?.setParameter(name: "Left_Image", value: .textureResource(texture))

//                    try shader?.setParameter(name: "Color", value: self.toggle ? .color(.blue) : .color(.magenta))

                    cube?.model?.materials = [shader!]
                } catch let error {
                    print("Failed \(error)")
                }


                let uniformScale: Float = self.toggle ? 1.4 : 1.0
                scene.transform.scale = [uniformScale, uniformScale, uniformScale]
            }
        }.onAppear(perform: {
//            self.queue.async {
//                while (true) {
//                    let frame = self.virtualFriend.run_frame()
//                    let cgImage = rustVecToCGImage(frame.left)
//                    self.image = cgImage
//                }
//            }
        })
        Image(self.image, scale: 1.0, label: Text("Hi"))
        Button("Transfer") {
            self.activeImage = self.image
        }
        Toggle("Hi", isOn: $toggle)
    }
}

func rustVecToCGImage(_ vec: RustVec<UInt8>) -> CGImage {
    var data = [UInt8](repeating: 0, count: PIXEL_BYTE_COUNT)

    for i in 0..<PIXEL_COUNT {
        let value = vec[i]

        data[i * 3] = value
        data[i * 3 + 1] = 0
        data[i * 3 + 2] = 0
    }

    let colorspace = CGColorSpaceCreateDeviceRGB()
    let rgbData = CFDataCreate(nil, data, PIXEL_BYTE_COUNT)!
    let provider = CGDataProvider(data: rgbData)!
    return CGImage(width: PIXEL_WIDTH,
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
}

#Preview {
    EmuView()
}
