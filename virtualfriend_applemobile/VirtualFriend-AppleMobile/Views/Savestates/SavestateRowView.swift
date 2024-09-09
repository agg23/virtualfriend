//
//  SavestateRowView.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 9/7/24.
//

import SwiftUI

struct SavestateRowView: View {
    @Environment(\.colorScheme) private var colorScheme: ColorScheme

    @State private var unparsedSavestate: UnparsedSavestateWithUrl? = nil

    let datedUrl: DatedSavestateUrl
    let onTap: (FFIUnparsedSavestate) -> Void
    let timeFormatter: DateFormatter

    init(datedUrl: DatedSavestateUrl, onTap: @escaping (FFIUnparsedSavestate) -> Void) {
        self.datedUrl = datedUrl
        self.onTap = onTap

        let timeFormatter = DateFormatter()
        timeFormatter.dateStyle = .none
        timeFormatter.timeStyle = .medium
        self.timeFormatter = timeFormatter
    }

    var body: some View {
        Button {
            self.tap()
        } label: {
            HStack {
                if let unparsedSavestate = self.unparsedSavestate {
                    StereoManifestImageView(data: unparsedSavestate, generateImage: { savestate, ledColor in
                        FileEntry.image(from: savestate.savestate, color: ledColor)
                    }, onTap: self.tap, integerScaling: false)
                    // TODO: Change for visionOS
                    .frame(height: 50)
                } else {
                    // TODO: Proper placeholder
                    Color.clear
                }

                Text(self.timeFormatter.string(from: self.datedUrl.date))
            }
        }
        .tint(self.colorScheme == .light ? .black : .white)
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

    func tap() {
        if let savestate = self.unparsedSavestate?.savestate {
            self.onTap(savestate)
        }
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
