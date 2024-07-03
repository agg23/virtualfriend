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

    @State private var emulator: EmulatorStatus = .none

    @State private var controlVisibilityTimer: Timer?
    @State private var preventControlDismiss: Bool = false
    @State private var controlVisibility: Visibility = .hidden

    let controlsTimerDuration = 3.0

    let fileUrl: URL

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
        .onTapGesture {
            self.toggleVisibility()
        }
        .onChange(of: self.fileUrl, initial: true) { _, newValue in
            self.createEmulator(newValue)
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
        #if os(visionOS)
        let buttonPadding = 40.0
        #else
        let buttonPadding = 8.0
        #endif

        ZStack(alignment: .top) {
            // Clear does not get drawn on top of the StereoImageView in visionOS for some reason
            Color.white.opacity(0.0001)

            if self.controlVisibility == .visible {
                HStack {
                    Button {
                        self.resetTimer()

                        self.router.currentRoute = .main

                        if case .emulator(let emulator) = self.emulator {
                            emulator.shutdown()
                        }
                    } label: {
                        Label {
                            Text("Back to Library")
                        } icon: {
                            Image(systemName: Icon.back)
                        }
                    }
                    .help("Back to Library")
                    #if os(visionOS)
                    .padding([.leading, .top], buttonPadding)
                    #else
                    .padding(.leading, buttonPadding)
                    #endif

                    Spacer()

                    Button {
                        self.resetTimer()

                        self.restart()
                    } label: {
                        Label {
                            Text("Restart")
                        } icon: {
                            Image(systemName: Icon.restart)
                        }
                    }
                    .help("Restart")
                    #if os(visionOS)
                    .padding([.trailing, .top], buttonPadding)
                    #else
                    .padding(.trailing, buttonPadding)
                    #endif

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
//                .padding(.bottom, 16)
//                .background(.thickMaterial)
            }
        }
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
            let emulator = try Emulator(fileUrl: url)
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
            self.createEmulator(self.fileUrl)
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
