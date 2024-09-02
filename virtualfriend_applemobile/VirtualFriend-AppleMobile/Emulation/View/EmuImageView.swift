//
//  EmuImageView.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/9/24.
//

import SwiftUI

struct EmuImageView: View {
    @Environment(\.scenePhase) private var scenePhase
    @Environment(\.openWindow) var openWindow

    @LEDColor var ledColor
    @Enable3D var enable3D

    let emulator: Emulator

    init(emulator: Emulator) {
        self.emulator = emulator
    }

    var body: some View {
        StereoImageView(width: 384, height: 224, scale: 1.0, stereoImageChannel: self.emulator.stereoImageChannel, backgroundColor: self._ledColor.colorWrapper.$background, force2D: !self.enable3D)
            .onChange(of: self.scenePhase) { _, newPhase in
                if newPhase == .background {
                    // Stop emulation
                    self.emulator.shutdown()
                }
            }
            .onChange(of: self.ledColor, initial: true) { _, _ in
                self.emulator.color = self.ledColor
            }
            .task {
                // For some reason when transitioning between immersive mode, the destruction and recreation of EmuImageView will
                // cause onAppear/onDisappear to run out of order. Use .task to ensure order
                self.emulator.start()
            }
            .onDisappear {
                self.emulator.shutdown()
            }
    }
}
