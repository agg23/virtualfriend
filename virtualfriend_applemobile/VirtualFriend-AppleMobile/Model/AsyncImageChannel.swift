//
//  AsyncIDChannel.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 4/27/24.
//

import Foundation
import AsyncAlgorithms

struct AsyncImageChannel {
    let id = UUID()
    let channel: AsyncChannel<StereoImage>

    init() {
        self.channel = AsyncChannel<StereoImage>()
    }
}

extension AsyncImageChannel: Equatable {
    static func == (lhs: AsyncImageChannel, rhs: AsyncImageChannel) -> Bool {
        lhs.id == rhs.id
    }
}
