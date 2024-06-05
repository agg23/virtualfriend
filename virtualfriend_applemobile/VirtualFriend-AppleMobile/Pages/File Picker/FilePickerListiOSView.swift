//
//  FilePickerListiOSView.swift
//  VirtualFriend-AppleMobile
//
//  Created by Adam Gastineau on 6/5/24.
//

import SwiftUI

struct FilePickerListiOSView: View {
    @Environment(MainRouter.self) private var router

    let files: [FileEntryWithManifest]

    var body: some View {
        List(self.files, id: \.entry.id) { file in
            Button {
                self.router.currentRoute = .emulator(url: file.entry.url)
            } label: {
                HStack {
                    StereoManifestImageView(entry: file, integerScaling: false)
                        .frame(height: 50)

                    VStack(alignment: .leading) {
                        Text(self.title(from: file))
                            .font(.title3)
                            .foregroundStyle(.primary)
                            .lineLimit(1)

                        if let metadata = file.manifest?.metadata {
                            Text(metadata.publisher.toString() + " " + metadata.year.toString())
                                .lineLimit(1)
                                .foregroundStyle(.secondary)
                        } else {
                            // Placeholder
                            // TODO: There should be something better that can be done here
                            Text(" ")
                        }
                    }
                }
            }
            .buttonStyle(BlackTextButtonStyle())
        }
    }

    func title(from entry: FileEntryWithManifest) -> String {
        entry.manifest?.metadata?.title.toString() ?? entry.entry.url.deletingPathExtension().lastPathComponent
    }
}

#Preview {
    FilePickerListiOSView(files: [.init(entry: .init(url: URL(string: "file://foo")!, hash: nil), manifest: nil), .init(entry: .init(url: URL(string: "file://foo")!, hash: nil), manifest: nil), .init(entry: .init(url: URL(string: "file://foo")!, hash: nil), manifest: nil)])
        .environment(MainRouter())
}
