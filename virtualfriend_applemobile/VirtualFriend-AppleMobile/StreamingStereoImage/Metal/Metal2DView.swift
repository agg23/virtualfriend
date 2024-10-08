//
//  Metal2DView.swift
//  VirtualFriend-AppleMobile
//
//  Created by Adam Gastineau on 6/3/24.
//

import SwiftUI

#if os(iOS)
struct Metal2DView: UIViewRepresentable {
    typealias UIViewType = Metal2DUIView

    let stereoImageChannel: AsyncImageChannel
    let size: CGSize
    let integerScaling: Bool
    let backgroundColor: CGColor

    func makeCoordinator() -> MetalCoordinator {
        MetalCoordinator()
    }

    func makeUIView(context: Context) -> Metal2DUIView {
        Metal2DUIView()
    }

    func updateUIView(_ uiView: Metal2DUIView, context: Context) {
        uiView.expectedSize = self.size
        uiView.integerScaling = self.integerScaling
        uiView.metalBackgroundColor = self.backgroundColor
        context.coordinator.task?.cancel()
        context.coordinator.task = Task {
            for await image in self.stereoImageChannel.channel.buffer(policy: .bounded(1)) {
                if Task.isCancelled {
                    return
                }

                uiView.image = image.left
            }
        }
    }

    class MetalCoordinator: NSObject {
        var task: Task<(), Error>?
    }
}
#endif
