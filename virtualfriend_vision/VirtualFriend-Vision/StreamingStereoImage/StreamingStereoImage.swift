//
//  StreamingStereoImage.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/24/24.
//

import CoreImage
import Combine

class StreamingStereoImage: ObservableObject {
    @Published var image: StereoImage

    init(image: StereoImage) {
        self.image = image
    }

    func update(left: CIImage, right: CIImage) {
        self.image = StereoImage(left: left, right: right)
    }
}

struct StereoImage {
    let left: CIImage?
    let right: CIImage?
}
