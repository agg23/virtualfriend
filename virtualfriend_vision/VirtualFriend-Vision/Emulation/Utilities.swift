//
//  Utilities.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/27/24.
//

import Foundation
import CoreImage

let PIXEL_WIDTH = 384
let PIXEL_HEIGHT = 224
let PIXEL_COUNT = PIXEL_WIDTH * PIXEL_HEIGHT
let PIXEL_BYTE_COUNT = PIXEL_COUNT * 4

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

    return CIImage(bitmapData: bitmapData, bytesPerRow: PIXEL_WIDTH * 4, size: .init(width: PIXEL_WIDTH, height: PIXEL_HEIGHT), format: .RGBA8, colorSpace: CGColorSpace(name: CGColorSpace.sRGB)!)
}

