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

    let context = CIContext()

    @StateObject var streamingStereoImage = StreamingStereoImage(image: StereoImage(left: nil, right: nil))

    init() {
        self.queue = DispatchQueue(label: "emu", qos: .userInteractive)

        let url = Bundle.main.url(forResource: "Mario's Tennis (Japan, USA)", withExtension: "vb")

        guard let url = url else {
            assertionFailure("Could not find embedded ROM")
            self.virtualFriend = nil
            return
        }

        self.virtualFriend = VirtualFriend(url.path(percentEncoded: false))
    }

    var body: some View {
        StreamingStereoImageView(width: 384, height: 224, stereoImage: self.streamingStereoImage)
        .onAppear(perform: {
            self.queue.async {
                while (true) {
                    let frame = self.virtualFriend.run_frame()
                    let leftImage = rustVecToCIImage(frame.left)
                    let rightImage = rustVecToCIImage(frame.right)

                    // TODO: This should be flipped by Metal, not the CPU
                    let leftTransformedImage = leftImage.transformed(by: .init(scaleX: 1, y: -1))
                    let rightTransformedImage = rightImage.transformed(by: .init(scaleX: 1, y: -1))

                    DispatchQueue.main.async {
                        self.streamingStereoImage.update(left: leftTransformedImage, right: rightTransformedImage)
                    }
                }
            }
        })
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
