//
//  SavestateRowView.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 9/7/24.
//

import SwiftUI

struct SavestateRowView: View {
    @State private var unparsedSavestate: UnparsedSavestateWithUrl? = nil

    let datedUrl: DatedSavestateUrl

    var body: some View {
        HStack {
            if let unparsedSavestate = self.unparsedSavestate {
                StereoManifestImageView(data: unparsedSavestate, generateImage: { savestate, ledColor in
                    FileEntry.image(from: savestate.savestate, color: ledColor)
                })
            } else {
                // TODO: Proper placeholder
                Color.clear
            }

            Text(datedUrl.date.ISO8601Format())
        }
        .onAppear {
            self.loadPreview()
        }
    }

    func loadPreview() {
        guard let savestate = load_savestate(self.datedUrl.url.path(percentEncoded: false)) else {
            return
        }

        self.unparsedSavestate = UnparsedSavestateWithUrl(savestate: savestate, url: self.datedUrl.url)
    }
}

struct UnparsedSavestateWithUrl {
    let savestate: FFIUnparsedSavestate
    let url: URL
}

extension UnparsedSavestateWithUrl: Equatable {
    static func == (lhs: UnparsedSavestateWithUrl, rhs: UnparsedSavestateWithUrl) -> Bool {
        lhs.url == rhs.url
    }
}
