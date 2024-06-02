//
//  StereoImageScene.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/28/24.
//

import Foundation
import RealityKit

#if os(visionOS)
class StereoImageMaterial {
    static let shared = StereoImageMaterial()

    let task: Task<ShaderGraphMaterial?, Error>

    var material: ShaderGraphMaterial? {
        get async {
            guard let material = try? await self.task.value else {
                fatalError("Could not load material")
            }

            return material
        }
    }

    private init() {
        self.task = Task {
            return try? await ShaderGraphMaterial(named: "/Root/SideBySideStereoRenderMaterial", from: "StereoImageMaterial")
        }
    }
}
#endif
