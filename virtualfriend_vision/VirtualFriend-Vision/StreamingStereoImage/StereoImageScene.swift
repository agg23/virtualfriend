//
//  StereoImageScene.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/28/24.
//

import Foundation
import RealityKit
import VBStereoRenderRealityKit

class StereoImageScene {
    static let shared = StereoImageScene()

    let task: Task<Entity?, Error>

    var scene: Entity? {
        get async {
            guard let scene = try? await self.task.value else {
                fatalError("Could not load scene")
            }

            return await scene.clone(recursive: true)
        }
    }

    private init() {
        self.task = Task {
            return try? await Entity(named: "Scene", in: vBStereoRenderRealityKitBundle)
        }
    }
}
