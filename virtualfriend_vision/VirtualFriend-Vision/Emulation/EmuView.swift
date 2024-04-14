//
//  EmuView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/21/24.
//

import SwiftUI
import RealityKit
import VBStereoRenderRealityKit
import AsyncAlgorithms

struct EmuView: View {
    let fileUrl: URL

    @State var emulator: Emulator?

    var body: some View {
        ZStack {
            // Black to surround the view and pad out the window AR
            Color.black
                .ignoresSafeArea()

            if let emulator = self.emulator {
                EmuContentView(emulator: emulator)
            } else {
                Text("Could not start emulator")
            }
        }
        .onChange(of: self.fileUrl, initial: true) { _, newValue in
            self.emulator = Emulator(fileUrl: newValue)
        }
    }
}

private struct EmuContentView: View {
    let emulator: Emulator

    var body: some View {
        StereoImageView(width: 384, height: 224, scale: 1.0, stereoImageChannel: self.emulator.stereoImageChannel)
            .onAppear {
                self.emulator.start()
            }
            .onDisappear {
                self.emulator.stop()
            }
    }
}
