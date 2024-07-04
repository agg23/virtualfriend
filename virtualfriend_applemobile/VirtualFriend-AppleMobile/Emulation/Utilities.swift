//
//  Utilities.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/27/24.
//

import Foundation
import CoreImage
import UIKit

private let PIXEL_WIDTH = 384
private let PIXEL_HEIGHT = 224
private let PIXEL_COUNT = PIXEL_WIDTH * PIXEL_HEIGHT
private let PIXEL_BYTE_COUNT = PIXEL_COUNT * 4

private let context = CIContext()

/// Cache color mixing calculations to speed up generation of rendered images
struct VBColor {
    private var red: [UInt8]
    private var green: [UInt8]
    private var blue: [UInt8]

    init(foregroundColor: CGColor, backgroundColor: CGColor) {
        let highlightComponents = foregroundColor.components!
        let backgroundComponents = backgroundColor.components!

        self.red = [UInt8](repeating: 0, count: 256)
        self.green = [UInt8](repeating: 0, count: 256)
        self.blue = [UInt8](repeating: 0, count: 256)

        for i in 0..<256 {
            let percent = Double(i) / 255.0

            self.red[i]   = UInt8(truncating: (backgroundComponents[0] + (highlightComponents[0] - backgroundComponents[0]) * percent) * 255.0 as NSNumber)
            self.green[i] = UInt8(truncating: (backgroundComponents[1] + (highlightComponents[1] - backgroundComponents[1]) * percent) * 255.0 as NSNumber)
            self.blue[i]  = UInt8(truncating: (backgroundComponents[2] + (highlightComponents[2] - backgroundComponents[2]) * percent) * 255.0 as NSNumber)
        }
    }

    func get(value: UInt8) -> (UInt8, UInt8, UInt8) {
        (self.red[Int(value)], self.green[Int(value)], self.blue[Int(value)])
    }
}

extension VBColor: Equatable {
    
}

extension RustVec<UInt8> {
    func ciImage(color: VBColor) -> CIImage {
        var bytes = [UInt8](repeating: 0, count: PIXEL_BYTE_COUNT)

        for i in 0..<PIXEL_COUNT {
            let value = self[i]

            let (red, green, blue) = color.get(value: value)

            bytes[i * 4 + 0] = red
            bytes[i * 4 + 1] = green
            bytes[i * 4 + 2] = blue
            // Alpha
            bytes[i * 4 + 3] = 255
        }

        let bitmapData = Data(bytes)

        return CIImage(bitmapData: bitmapData, bytesPerRow: PIXEL_WIDTH * 4, size: .init(width: PIXEL_WIDTH, height: PIXEL_HEIGHT), format: .RGBA8, colorSpace: CGColorSpace(name: CGColorSpace.displayP3)!)
    }

    func uiImage(color: VBColor) -> UIImage {
        let ciImage = self.ciImage(color: color)
        context.createCGImage(ciImage, from: .init(x: 0, y: 0, width: PIXEL_WIDTH, height: PIXEL_HEIGHT))

        // Going directly from CIImage to UIImage doesn't seem to work
        return UIImage(cgImage: context.createCGImage(ciImage, from: .init(x: 0, y: 0, width: PIXEL_WIDTH, height: PIXEL_HEIGHT))!)
    }
}
