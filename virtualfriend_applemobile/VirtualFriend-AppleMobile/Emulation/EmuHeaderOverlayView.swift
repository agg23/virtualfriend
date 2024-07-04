//
//  EmuHeaderOverlayView.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/4/24.
//

import SwiftUI

struct EmuHeaderOverlayView: View {
    let title: String

    let resetTimer: () -> Void
    let onBack: () -> Void
    let onRestart: () -> Void

    var body: some View {
        // Attempt to match NavigationBar padding
        #if os(visionOS)
        let buttonPadding = 20.0
        let bottomPadding = 16.0
        #else
        let buttonPadding = 8.0
        let bottomPadding = 8.0
        #endif

        HStack {
            HStack {
                Button {
                    self.resetTimer()

                    self.onBack()
                } label: {
                    Label {
                        Text("Back to Library")
                    } icon: {
                        Image(systemName: Icon.back)
                    }
                }
                .help("Back to Library")
                .padding(.leading, buttonPadding)

                Spacer()

                Text(self.title)
                    #if os(visionOS)
                    .font(.title)
                    #else
                    .font(.title3)
                    #endif
                    .lineLimit(1)
                    .truncationMode(.tail)
                    .foregroundColor(.white)

                Spacer()

                Button {
                    self.resetTimer()

                    self.onRestart()
                } label: {
                    Label {
                        Text("Restart")
                    } icon: {
                        Image(systemName: Icon.restart)
                    }
                }
                .help("Restart")
                .padding(.trailing, buttonPadding)
            }
        }
        .symbolRenderingMode(.hierarchical)
        .labelStyle(.iconOnly)
        .buttonBorderShape(.circle)
        .controlSize(.large)
        #if !os(visionOS)
        .tint(.white)
        .symbolVariant(.circle.fill)
        .font(.largeTitle)
        #endif
        .padding(.bottom, bottomPadding)
        .padding(.top, buttonPadding)
        #if os(visionOS)
        // Vision looks bad with the (white) .secondary, so we use the same color as our details view
        .background(Color.black.opacity(0.4))
        #else
        // We insert .secondary background so we can ensure the control buttons are visible over the user configuable background color
        .background(.secondary)
        #endif
    }
}
