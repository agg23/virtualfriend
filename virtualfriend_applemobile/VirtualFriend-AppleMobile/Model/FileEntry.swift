//
//  FileEntry.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 4/27/24.
//

import Foundation
import SwiftUI

struct FileEntry {
    let url: URL
    let hash: String?

    var manifest: FFIManifest? {
        get {
            guard let manifest = self.getManifest() else {
                return nil
            }

            return manifest
        }
    }

    var withManifest: FileEntryWithManifest {
        get {
            FileEntryWithManifest(entry: self, manifest: self.manifest)
        }
    }

    private func getManifest() -> FFIManifest? {
        guard let hash = self.hash else {
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

    static func image(from manifest: FFIManifest, color: VBColor) -> StereoImage {
        let left = manifest.left_frame.ciImage(color: color)
        let right = manifest.right_frame.ciImage(color: color)

        return StereoImage(left: left, right: right)
    }

    static func image(from savestate: FFIUnparsedSavestate, color: VBColor) -> StereoImage {
        let left = savestate.left_frame.ciImage(color: color)
        let right = savestate.right_frame.ciImage(color: color)

        return StereoImage(left: left, right: right)
    }
}

extension FileEntry: Identifiable {
    var id: URL {
        get {
            return self.url
        }
    }
}

struct FileEntryWithManifest {
    let entry: FileEntry
    let manifest: FFIManifest?

    var title: String {
        self.manifest?.metadata?.title.toString() ?? self.entry.url.deletingPathExtension().lastPathComponent
    }
}

extension FileEntryWithManifest: Hashable {
    static func == (lhs: FileEntryWithManifest, rhs: FileEntryWithManifest) -> Bool {
        lhs.entry.id == rhs.entry.id
    }

    func hash(into hasher: inout Hasher) {
        hasher.combine(self.entry.url)
    }
}
