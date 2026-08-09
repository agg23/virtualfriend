//
//  StereoImageScene.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/28/24.
//

import Foundation
import RealityKit

#if os(visionOS)
@MainActor
class StereoImageMaterial {
    static let shared = StereoImageMaterial()

    private let task: Task<ShaderGraphMaterial?, Never>

    private init() {
        self.task = Task {
            do {
                return try await ShaderGraphMaterial(named: "/Root/SideBySideStereoRenderMaterial", from: "StereoImageMaterial")
            } catch {
                print("Could not load stereo material: \(error)")

                return nil
            }
        }
    }

    var material: ShaderGraphMaterial? {
        get async {
            await self.task.value
        }
    }
}
#endif
