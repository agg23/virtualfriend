//
//  EmuView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/21/24.
//

import SwiftUI
import RealityKit
import AsyncAlgorithms

enum EmulatorStatus {
    case emulator(_ emulator: Emulator)
    case error(_ message: String)
    case none
}

struct EmuView: View {
    @Environment(\.scenePhase) private var scenePhase

    @EyeSeparation private var separation

    @EnableSound private var enableSound
    @Enable3D private var enable3D

    @State private var emulator: EmulatorStatus = .none
    @State private var controller: EmuController = EmuController()

    let fileEntry: FileEntryWithManifest

    var body: some View {
        EmuContentView(emulator: self.emulator, controller: self.controller, title: self.fileEntry.title, onRestart: self.restart)
            .persistentSystemOverlays(.hidden)
            .onChange(of: self.fileEntry, initial: true) { _, newValue in
                self.createEmulator(newValue.entry.url)
            }
            .onChange(of: self.scenePhase) { prevValue, newValue in
                guard case .emulator(let emulator) = self.emulator else {
                    return
                }

                switch newValue {
                case .active:
                    if prevValue != .active {
                        // We resumed activity. Start emulation
                        emulator.start()
                    }
                case .inactive:
                    fallthrough
                case .background:
                    emulator.shutdown()
                @unknown default:
                    print("Unknown scene \(newValue)")
                }
            }
            .onChange(of: self.separation) { _, newValue in
                if case .emulator(let emulator) = self.emulator {
                    // Invert separation range so more 3D is on the right
                    emulator.separation = newValue * -1
                }
            }
            .onChange(of: self.enableSound) { _, newValue in
                if case .emulator(let emulator) = self.emulator {
                    emulator.enableSound = newValue
                }
            }
    }

    func createEmulator(_ url: URL) {
        do {
            let emulator = try Emulator(fileUrl: url, controller: self.controller)
            emulator.separation = self.separation
            emulator.enableSound = self.enableSound
            self.emulator = .emulator(emulator)
        } catch {
            self.emulator = .error(error.localizedDescription)
        }
    }

    func restart() {
        if case .emulator(let emulator) = self.emulator {
            emulator.shutdown()
        }

        self.emulator = .none
        // TODO: Make Emulator stereoImageChannel updates cause rerenders
        DispatchQueue.main.asyncAfter(deadline: .now().advanced(by: .milliseconds(100)), execute: .init(block: {
            self.createEmulator(self.fileEntry.entry.url)
        }))
    }
}
