//
//  EmuView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/21/24.
//

import SwiftUI
import RealityKit
import AsyncAlgorithms

struct EmuView: View {
    private enum EmulatorStatus {
        case emulator(_ emulator: Emulator)
        case error(_ message: String)
        case none
    }

    @Environment(MainRouter.self) private var router
    @Environment(\.horizontalSizeClass) private var horizontalSizeClass
    @Environment(\.scenePhase) private var scenePhase

    @LEDBackgroundColor var ledBackgroundColor
    @EyeSeparation var separation

    @EnableSound var enableSound
    @Enable3D var enable3D

    @State private var controller: EmuController = EmuController()

    @State private var emulator: EmulatorStatus = .none

    @State private var controlVisibilityTimer: Timer?
    @State private var preventControlDismiss: Bool = false
    @State private var controlVisibility: Visibility = .hidden

    let controlsTimerDuration = 3.0

    let fileEntry: FileEntryWithManifest

    var body: some View {
        #if os(visionOS)
        let alignment = Alignment.center
        #else
        let alignment = if case .compact = self.horizontalSizeClass {
            // Portrait, put emulator at top
            Alignment.top
        } else {
            // Otherwise center
            Alignment.center
        }
        #endif

        let toastText = Binding<ToastText> {
            .content(text: self.controller.notification.text, icon: self.controller.notification.icon)
        } set: { text in
            if text == .none {
                self.controller.notification = .none
            }
        }

//        ToastWrapper(text: toastText) {
            ZStack(alignment: alignment) {
                // Background color to surround the view and pad out the window AR
                self.ledBackgroundColor
                    .ignoresSafeArea()

                Group {
                    switch self.emulator {
                    case .emulator(let emulator):
                        EmuContentView(emulator: emulator, controlVisibility: self.$controlVisibility, preventControlDismiss: self.$preventControlDismiss)
                            .padding(.vertical, 16)
                    case .error(let message):
                        VStack(alignment: .center) {
                            Text("Could not start emulator")

                            Text(message)
                        }
                    case .none:
                        EmptyView()
                    }
                }
                #if os(visionOS)
                // Add additional spacing around render frame to prevent corner from clipping off of rounded corner
                .padding(8)
                #endif
            }
//        }
        .onTapGesture {
            self.toggleVisibility()
        }
        #if !os(visionOS)
        .overlay {
            EmuControllerView(controller: self.controller)
        }
        #endif
        .overlay {
            self.controlsOverlay
        }
        #if os(visionOS)
        .ornament(visibility: self.controlVisibility, attachmentAnchor: .scene(.bottom)) {
            if self.enable3D {
                self.eyeSeparationOverlay
            }
        }
        #endif
        .onChange(of: self.fileEntry, initial: true) { _, newValue in
            self.createEmulator(newValue.entry.url)
        }
        .onChange(of: self.preventControlDismiss) { _, newValue in
            if newValue {
                self.clearTimer()
            } else {
                self.resetTimer()
            }
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

    @ViewBuilder
    var controlsOverlay: some View {
        ZStack(alignment: .top) {
            // Clear does not get drawn on top of the StereoImageView in visionOS for some reason
            Color.white.opacity(0.0001)
                .allowsHitTesting(false)

            if self.controlVisibility == .visible {
                EmuHeaderOverlayView(title: self.fileEntry.title) {
                    self.resetTimer()
                } onBack: {
                    self.router.currentRoute = .main

                    if case .emulator(let emulator) = self.emulator {
                        emulator.shutdown()
                    }
                } onRestart: {
                    self.restart()
                }
            }
        }
        #if os(visionOS)
        // Should be the exact window corner radius
        .clipShape(.rect(cornerRadius: 46.0))
        #endif
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    #if os(visionOS)
    @ViewBuilder
    var eyeSeparationOverlay: some View {
        VStack {
            Text(String(format: "Eye Separation: %.1f", self.separation))
                .font(.title3)

            Slider(value: self.$separation, in: -4...4, step: 0.5, label: {
                Text("Separation")
            }, minimumValueLabel: {
                // Less 3D
                Text("-4")
            }, maximumValueLabel: {
                // More 3D
                Text("4")
            }) { editing in
                self.preventControlDismiss = editing
            }
        }
        .padding(24)
        .frame(width: 600)
        .glassBackgroundEffect()
    }
    #endif

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

    func resetTimer() {
        self.controlVisibilityTimer?.invalidate()
        self.controlVisibilityTimer = Timer.scheduledTimer(withTimeInterval: self.controlsTimerDuration, repeats: false, block: { _ in
            withAnimation {
                self.controlVisibility = .hidden
            }
        })
    }

    func clearTimer() {
        self.controlVisibilityTimer?.invalidate()
        self.controlVisibilityTimer = nil
    }

    func toggleVisibility() {
        if self.controlVisibility == .visible {
            self.clearTimer()
        } else {
            self.resetTimer()
        }

        withAnimation {
            self.controlVisibility = self.controlVisibility != .visible ? .visible : .hidden
        }
    }

    func rebuildControllers() {

    }
}

private struct EmuContentView: View {
    @Environment(\.scenePhase) private var scenePhase
    @Environment(\.openWindow) var openWindow

    @LEDColor var ledColor
    @Enable3D var enable3D

    let emulator: Emulator
    @Binding var controlVisibility: Visibility
    @Binding var preventControlDismiss: Bool

    init(emulator: Emulator, controlVisibility: Binding<Visibility>, preventControlDismiss: Binding<Bool>) {
        self.emulator = emulator
        self._controlVisibility = controlVisibility
        self._preventControlDismiss = preventControlDismiss
    }

    var body: some View {
        StereoImageView(width: 384, height: 224, scale: 1.0, stereoImageChannel: self.emulator.stereoImageChannel, backgroundColor: self._ledColor.colorWrapper.$background, force2D: !self.enable3D)
            .onChange(of: self.scenePhase) { _, newPhase in
                if newPhase == .background {
                    // Stop emulation
                    self.emulator.shutdown()
                }
            }
            .onChange(of: self.ledColor) { _, _ in
                self.emulator.color = self.ledColor
            }
            .onAppear {
                self.emulator.color = self.ledColor

                self.emulator.start()
            }
            .onDisappear {
                self.emulator.shutdown()
            }
    }
}
