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

    @LEDBackgroundColor var ledBackgroundColor;

    @State private var emulator: EmulatorStatus = .none

    @State private var controlVisibilityTimer: Timer?
    @State private var preventControlDismiss: Bool = false
    @State private var controlVisibility: Visibility = .hidden

    let controlsTimerDuration = 3.0

    let fileUrl: URL

    var body: some View {
        ZStack {
            // Background color to surround the view and pad out the window AR
            self.ledBackgroundColor
                .ignoresSafeArea()
                // Default system corner radius
                #if os(visionOS)
                // Make window appear to be rounded
                .clipShape(.rect(cornerRadius: 56))
                #endif


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
        .overlay {
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
                            self.router.currentRoute = .main
                        } label: {
                            Label {
                                Text("Back")
                            } icon: {
                                Image(systemName: Icon.back)
                            }
                        }
                        .help("Back")
                        .labelStyle(.iconOnly)
                        .buttonBorderShape(.circle)
                        .controlSize(.large)
                        .padding([.leading, .top], buttonPadding)

                        Spacer()

                        Button {
                            self.restart()
                        } label: {
                            Label {
                                Text("Restart")
                            } icon: {
                                Image(systemName: Icon.restart)
                            }
                        }
                        .help("Restart")
                        .labelStyle(.iconOnly)
                        .buttonBorderShape(.circle)
                        .controlSize(.large)
                        .padding([.trailing, .top], buttonPadding)

                    }
                }
            }
            .frame(maxWidth: /*@START_MENU_TOKEN@*/.infinity/*@END_MENU_TOKEN@*/, maxHeight: .infinity)
        }
        #if os(visionOS)
        .ornament(visibility: self.controlVisibility, attachmentAnchor: .scene(.bottom)) {
            VStack {
                // Add spacing between main window and ornament content to allow for the window resizer
                Color.clear.frame(height: 180.0)

                VStack {
                    HStack {
                        Text("Separation")

                        Slider(value: self.$separation, in: -5...5, step: 0.01, label: {
                            Text("Separation")
                        }, minimumValueLabel: {
                            Text("-5")
                        }, maximumValueLabel: {
                            Text("5")
                        }) { editing in
                            self.preventControlDismiss = editing
                        }
                    }
                    Text("\(self.separation)")

                    Toggle("Enable sound", isOn: self.$sound)
                    Text("Note: Sound is extremely beta and likely broken")
                        .font(.footnote)
                }
                .padding(24)
                .frame(width: 600)
                .glassBackgroundEffect()
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
    }

    func createEmulator(_ url: URL) {
        do {
            let emulator = try Emulator(fileUrl: url)
            self.emulator = .emulator(emulator)
        } catch {
            self.emulator = .error(error.localizedDescription)
        }
    }

    func restart() {
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

    @LEDBackgroundColor var ledBackgroundColor;
    @LEDForegroundColor var ledForegroundColor;

    let emulator: Emulator
    @Binding var controlVisibility: Visibility
    @Binding var preventControlDismiss: Bool

    @State private var separation: Double = 0.0
    @State private var sound: Bool = false

    init(emulator: Emulator, controlVisibility: Binding<Visibility>, preventControlDismiss: Binding<Bool>) {
        self.emulator = emulator
        self._controlVisibility = controlVisibility
        self._preventControlDismiss = preventControlDismiss
    }

    var body: some View {
        StereoImageView(width: 384, height: 224, scale: 1.0, stereoImageChannel: self.emulator.stereoImageChannel)
            .onChange(of: self.scenePhase) { _, newPhase in
                if newPhase == .background {
                    // Stop emulation
                    self.emulator.shutdown()
                }
            }
            .onChange(of: self.sound, { _, newValue in
                self.emulator.enableSound(newValue)
            })
            .onChange(of: self.ledBackgroundColor) { _, _ in
                self.emulator.set(foregroundColor: self.ledForegroundColor.rawCGColor, backgroundColor: self.ledBackgroundColor.rawCGColor)
            }
            .onChange(of: self.ledForegroundColor) { _, _ in
                self.emulator.set(foregroundColor: self.ledForegroundColor.rawCGColor, backgroundColor: self.ledBackgroundColor.rawCGColor)
            }
            .onAppear {
                self.emulator.set(foregroundColor: self.ledForegroundColor.rawCGColor, backgroundColor: self.ledBackgroundColor.rawCGColor)

                self.emulator.separation = self.$separation
                self.emulator.start()
            }
            .onDisappear {
                self.emulator.shutdown()
            }
    }
}
