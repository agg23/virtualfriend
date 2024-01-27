//
//  FilePickerView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/26/24.
//

import SwiftUI
import RealityKit

struct FilePickerView: View {
    @State var toggle: Bool

    let stereoImage: StreamingStereoImage

    init() {
        self.toggle = false

        let image = UIImage(named: "Blank Image")!

        let ciImage = CIImage(cgImage: image.cgImage!)

        self.stereoImage = StreamingStereoImage(image: StereoImage(left: ciImage, right: ciImage))
    }

    var body: some View {
        Grid {
            ForEach(0..<3) { _ in
                GridRow {
                    StreamingStereoImageView(width: 384, height: 224, stereoImage: stereoImage, zPosition: -0.18, scale: 0.9)
                    StreamingStereoImageView(width: 384, height: 224, stereoImage: stereoImage, zPosition: -0.18, scale: 0.9)
                    StreamingStereoImageView(width: 384, height: 224, stereoImage: stereoImage, zPosition: -0.18, scale: 0.9)
                }
            }
        }

            VStack (spacing: 12) {
                Text("Test")
                Toggle(isOn: $toggle, label: {
                    Text("Toggle")
                })
            }
            .frame(width: 600)
            .padding(36)
            .glassBackgroundEffect()
    }
}

#Preview {
    FilePickerView()
}
