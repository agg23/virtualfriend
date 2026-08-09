//
//  Utilities.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/27/24.
//

import Foundation
import CoreImage
import UIKit
import Accelerate

private let PIXEL_WIDTH = 384
private let PIXEL_HEIGHT = 224
private let PIXEL_COUNT = PIXEL_WIDTH * PIXEL_HEIGHT
private let PIXEL_BYTE_COUNT = PIXEL_COUNT * 4

private let context = CIContext()

/// Cache color mixing calculations to speed up generation of rendered images
struct VBColor {
    /// Pixel per brightness value, in the layout `vImageLookupTable_Planar8toPlanar24` expects with RGB as bytes 1, 2, and 3. Byte 0 is unused
    private var pixels: [UInt32]

    init(foregroundColor: CGColor, backgroundColor: CGColor) {
        let highlightComponents = foregroundColor.components!
        let backgroundComponents = backgroundColor.components!

        self.pixels = [UInt32](repeating: 0, count: 256)

        for i in 0..<256 {
            let percent = Double(i) / 255.0

            let red   = UInt32(truncating: (backgroundComponents[0] + (highlightComponents[0] - backgroundComponents[0]) * percent) * 255.0 as NSNumber)
            let green = UInt32(truncating: (backgroundComponents[1] + (highlightComponents[1] - backgroundComponents[1]) * percent) * 255.0 as NSNumber)
            let blue  = UInt32(truncating: (backgroundComponents[2] + (highlightComponents[2] - backgroundComponents[2]) * percent) * 255.0 as NSNumber)

            // 0, R, G, B
            self.pixels[i] = red << 8 | green << 16 | blue << 24
        }
    }

    func withPixelTable<T>(_ body: (UnsafePointer<UInt32>) -> T) -> T {
        self.pixels.withUnsafeBufferPointer { buffer in
            body(buffer.baseAddress!)
        }
    }
}

extension VBColor: Equatable {
    
}

extension RustVec<UInt8> {
    func ciImage(color: VBColor) -> CIImage {
        var bitmapData = Data(count: PIXEL_BYTE_COUNT)

        // Map to RGB, then add alpha in final step
        var rgbData = Data(count: PIXEL_COUNT * 3)

        withExtendedLifetime(self) {
            var source = vImage_Buffer(data: UnsafeMutableRawPointer(mutating: self.as_ptr()), height: vImagePixelCount(PIXEL_HEIGHT), width: vImagePixelCount(PIXEL_WIDTH), rowBytes: PIXEL_WIDTH)

            rgbData.withUnsafeMutableBytes { rgbRaw in
                var rgb = vImage_Buffer(data: rgbRaw.baseAddress, height: vImagePixelCount(PIXEL_HEIGHT), width: vImagePixelCount(PIXEL_WIDTH), rowBytes: PIXEL_WIDTH * 3)

                color.withPixelTable { table in
                    _ = vImageLookupTable_Planar8toPlanar24(&source, &rgb, table, vImage_Flags(kvImageDoNotTile))
                }

                bitmapData.withUnsafeMutableBytes { rgbaRaw in
                    var rgba = vImage_Buffer(data: rgbaRaw.baseAddress, height: vImagePixelCount(PIXEL_HEIGHT), width: vImagePixelCount(PIXEL_WIDTH), rowBytes: PIXEL_WIDTH * 4)

                    _ = vImageConvert_RGB888toRGBA8888(&rgb, nil, 255, &rgba, false, vImage_Flags(kvImageDoNotTile))
                }
            }
        }

        return CIImage(bitmapData: bitmapData, bytesPerRow: PIXEL_WIDTH * 4, size: .init(width: PIXEL_WIDTH, height: PIXEL_HEIGHT), format: .RGBA8, colorSpace: CGColorSpace(name: CGColorSpace.displayP3)!)
    }

    func uiImage(color: VBColor) -> UIImage {
        let ciImage = self.ciImage(color: color)
        context.createCGImage(ciImage, from: .init(x: 0, y: 0, width: PIXEL_WIDTH, height: PIXEL_HEIGHT))

        // Going directly from CIImage to UIImage doesn't seem to work
        return UIImage(cgImage: context.createCGImage(ciImage, from: .init(x: 0, y: 0, width: PIXEL_WIDTH, height: PIXEL_HEIGHT))!)
    }
}
