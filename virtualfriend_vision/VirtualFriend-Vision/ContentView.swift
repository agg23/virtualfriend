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

    var image = UIImage(named: "Left")!

    var body: some View {
        VStack {
//            RealityView { content in
//                // Add the initial RealityKit content
//                if let scene = try? await Entity(named: "Scene", in: vBStereoRenderRealityKitBundle) {
//                    content.add(scene)
//                }
//            } update: { content in
//                // Update the RealityKit content when SwiftUI state changes
//                if let scene = content.entities.first {
//                    let cube = scene.findEntity(named: "HoldingCube") as? ModelEntity
//                    var shader = cube?.model?.materials.first as? ShaderGraphMaterial
//
//                    do {
//                        let texture = try TextureResource.generate(from: image.cgImage!, withName: "leftTexture", options: TextureResource.CreateOptions.init(semantic: .raw))
//
//                        try shader?.setParameter(name: "Left_Image", value: .textureResource(texture))
//
//                        cube?.model?.materials = [shader!]
//                    } catch let error {
//                        print("Failed \(error)")
//                    }
//                    let uniformScale: Float = enlarge ? 1.4 : 1.0
//                    scene.transform.scale = [uniformScale, uniformScale, uniformScale]
//                }
//            }
//            .gesture(TapGesture().targetedToAnyEntity().onEnded { _ in
//                enlarge.toggle()
//            })

            VStack (spacing: 12) {
                EmuView()
                Toggle("Enlarge RealityView Content", isOn: $enlarge)
                    .font(.title)

                Toggle("Show ImmersiveSpace", isOn: $showImmersiveSpace)
                    .font(.title)
            }
            .frame(width: 360)
            .padding(36)
            .glassBackgroundEffect()

            Image(uiImage: image)
        }
//        .onChange(of: showImmersiveSpace) { _, newValue in
//            Task {
//                if newValue {
//                    switch await openImmersiveSpace(id: "ImmersiveSpace") {
//                    case .opened:
//                        immersiveSpaceIsShown = true
//                    case .error, .userCancelled:
//                        fallthrough
//                    @unknown default:
//                        immersiveSpaceIsShown = false
//                        showImmersiveSpace = false
//                    }
//                } else if immersiveSpaceIsShown {
//                    await dismissImmersiveSpace()
//                    immersiveSpaceIsShown = false
//                }
//            }
//        }
    }
}

#Preview(windowStyle: .volumetric) {
    ContentView()
}
