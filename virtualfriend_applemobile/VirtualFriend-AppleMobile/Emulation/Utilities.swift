//
//  Utilities.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/27/24.
//

import Foundation
import CoreImage

private let PIXEL_WIDTH = 384
private let PIXEL_HEIGHT = 224
private let PIXEL_COUNT = PIXEL_WIDTH * PIXEL_HEIGHT
private let PIXEL_BYTE_COUNT = PIXEL_COUNT * 4

extension RustVec<UInt8> {
    func ciImage(foregroundColor: CGColor, backgroundColor: CGColor) -> CIImage {
        let highlightComponents = foregroundColor.components!
        let backgroundComponents = backgroundColor.components!

        var bytes = [UInt8](repeating: 0, count: PIXEL_BYTE_COUNT)

        for i in 0..<PIXEL_COUNT {
            let value = self[i]

            let percent = Double(value) / 255.0

            bytes[i * 4] = UInt8(truncating: (backgroundComponents[0] + (highlightComponents[0] - backgroundComponents[0]) * percent) * 255.0 as NSNumber)
            bytes[i * 4 + 1] = UInt8(truncating: (backgroundComponents[1] + (highlightComponents[1] - backgroundComponents[1]) * percent) * 255.0 as NSNumber)
            bytes[i * 4 + 2] = UInt8(truncating: (backgroundComponents[2] + (highlightComponents[2] - backgroundComponents[2]) * percent) * 255.0 as NSNumber)
            // Alpha
            bytes[i * 4 + 3] = 255
        }

        let bitmapData = Data(bytes)

        return CIImage(bitmapData: bitmapData, bytesPerRow: PIXEL_WIDTH * 4, size: .init(width: PIXEL_WIDTH, height: PIXEL_HEIGHT), format: .RGBA8, colorSpace: CGColorSpace(name: CGColorSpace.sRGB)!)
    }
}
