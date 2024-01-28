//
//  FilePickerEntry.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/27/24.
//

import SwiftUI

struct FilePickerEntry: View {
    let stereoImage: StreamingStereoImage

    @Binding var fileUrl: URL
    @Binding var hash: String?

    init(fileUrl: Binding<URL>, hash: Binding<String?>) {
        self._fileUrl = fileUrl
        self._hash = hash

        let url = Bundle.main.url(forResource: "Mario's Tennis (Japan, USA)", withExtension: "vf")

        guard let url = url else {
            fatalError("Could not find embedded manifest")
        }

        guard let manifest = load_manifest(url.path(percentEncoded: false)) else {
            fatalError("Could not get manifest")
        }

        let left = rustVecToCIImage(manifest.left_frame)
        let right = rustVecToCIImage(manifest.right_frame)

        let leftTransformedImage = left.transformed(by: .init(scaleX: 1, y: -1))
        let rightTransformedImage = right.transformed(by: .init(scaleX: 1, y: -1))

        self.stereoImage = StreamingStereoImage(image: StereoImage(left: leftTransformedImage, right: rightTransformedImage))
    }

    var body: some View {
        VStack {
            StreamingStereoImageView(width: 384, height: 224, stereoImage: stereoImage, zPosition: -0.18, scale: 0.9)
                .frame(width: 400, height: 300)
            Text(hash ?? "")
//            Text("Mario's Tennis")
//                .font(.title)
//            Text("Nintendo")
        }
    }
}

#Preview {
    FilePickerEntry(fileUrl: .constant(URL(string: "hi")!), hash: .constant("foo"))
}
