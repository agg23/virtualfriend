//
//  ContentView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/9/24.
//

import SwiftUI
import RealityKit
import VBStereoRenderRealityKit

struct ContentView: View {

    @State private var enlarge = false
    @State private var showImmersiveSpace = false
    @State private var immersiveSpaceIsShown = false

    @Environment(\.openImmersiveSpace) var openImmersiveSpace
    @Environment(\.dismissImmersiveSpace) var dismissImmersiveSpace

    var body: some View {
        VStack {
            RealityView(make: { content in
                if let scene = try? await Entity(named: "Scene", in: vBStereoRenderRealityKitBundle) {
                    content.add(scene)
                }
                let cube = ModelEntity(mesh: .generateBox(size: 0.3))
                content.add(cube)
            })
        }
    }
}

#Preview(windowStyle: .volumetric) {
    ContentView()
}
