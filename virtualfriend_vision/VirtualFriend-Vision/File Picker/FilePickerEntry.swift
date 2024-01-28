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

    let metadata: FFIMetadata?

    init(fileUrl: Binding<URL>, hash: Binding<String?>) {
        self._fileUrl = fileUrl
        self._hash = hash

        guard let manifest = FilePickerEntry.getManifest(hash: hash.wrappedValue) else {
            let manifest = FilePickerEntry.getUnknownManifest()
            self.stereoImage = FilePickerEntry.manifestToImage(manifest)
//            self.stereoImage = StreamingStereoImage(image: StereoImage(left: nil, right: nil))
            self.metadata = nil
            return
        }

        self.stereoImage = FilePickerEntry.manifestToImage(manifest)
        self.metadata = manifest.metadata
    }

    var body: some View {
        VStack {
            StreamingStereoImageView(width: 384, height: 224, stereoImage: stereoImage, zPosition: -0.18, scale: 0.9)
                .frame(width: 400, height: 300)
            Text(self.metadata?.title.toString() ?? fileUrl.deletingPathExtension().lastPathComponent)
                .font(.title)
            if let metadata = self.metadata {
                Text(metadata.publisher.toString() + " " + metadata.year.toString())
            } else {
                // Placeholder
                // TODO: There should be something better that can be done here
                Text(" ")
            }
        }
    }

    static func getManifest(hash: String?) -> FFIManifest? {
        guard let hash = hash else {
            return nil
        }

        let manifests = Bundle.main.url(forResource: "manifests", withExtension: nil)
        guard let manifestFolderUrl = manifests?.appendingPathComponent(hash, conformingTo: .folder) else {
            return nil
        }

        guard let manifestUrl = FileManager.default.getFilesWithExtension(at: manifestFolderUrl, fileExtension: "vf").first else {
            return nil
        }

        guard let manifest = load_manifest(manifestUrl.path(percentEncoded: false)) else {
            return nil
        }

        return manifest
    }

    static func getUnknownManifest() -> FFIManifest {
        guard let manifests = Bundle.main.url(forResource: "manifests", withExtension: nil) else {
            fatalError("Could not find unknown game manifest")
        }

        let unknownUrl = manifests.appendingPathComponent("unknowngame.vf")

        guard let manifest = load_manifest(unknownUrl.path(percentEncoded: false)) else {
            fatalError("Could not find unknown game manifest")
        }

        return manifest
    }

    static func manifestToImage(_ manifest: FFIManifest) -> StreamingStereoImage {
        let left = rustVecToCIImage(manifest.left_frame)
        let right = rustVecToCIImage(manifest.right_frame)

        let leftTransformedImage = left.transformed(by: .init(scaleX: 1, y: -1))
        let rightTransformedImage = right.transformed(by: .init(scaleX: 1, y: -1))

        return StreamingStereoImage(image: StereoImage(left: leftTransformedImage, right: rightTransformedImage))
    }
}

#Preview {
    FilePickerEntry(fileUrl: .constant(URL(string: "hi")!), hash: .constant("foo"))
}
