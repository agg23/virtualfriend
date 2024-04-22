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
    let controlsTimerDuration = 3.0

    let fileUrl: URL

    @State private var emulator: Emulator?

    @State private var controlVisibilityTimer: Timer?
    @State private var controlVisibility: Visibility = .hidden

    var body: some View {
        ZStack {
            // Black to surround the view and pad out the window AR
            Color.black
                .ignoresSafeArea()

            if let emulator = self.emulator {
                EmuContentView(emulator: emulator, controlVisibility: self.$controlVisibility) {
                    self.emulator = nil
                    // TODO: Make Emulator stereoImageChannel updates cause rerenders
                    DispatchQueue.main.asyncAfter(deadline: .now().advanced(by: .milliseconds(100)), execute: .init(block: {
                        self.emulator = Emulator(fileUrl: self.fileUrl)
                    }))
                }
            } else {
                EmptyView()
            }
        }
        .onTapGesture {
            self.toggleVisibility()
        }
        .onChange(of: self.fileUrl, initial: true) { _, newValue in
            self.emulator = Emulator(fileUrl: newValue)
        }
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
    let emulator: Emulator
    @Binding var controlVisibility: Visibility

    let onRestart: () -> Void

    @State private var separation: Double = 0.0
    @State private var depth: Double = 1.0

    init(emulator: Emulator, controlVisibility: Binding<Visibility>, onRestart: @escaping () -> Void) {
        self.emulator = emulator
        self._controlVisibility = controlVisibility
        self.onRestart = onRestart
    }

    var body: some View {
        VStack {
            StereoImageView(width: 384, height: 224, scale: 1.0, stereoImageChannel: self.emulator.stereoImageChannel, depth: self.$depth)
                .overlay {
                    ZStack(alignment: .topTrailing) {
                        // Clear does not get drawn on top of the StereoImageView for some reason
                        Color.white.opacity(0.0001)

                        if self.controlVisibility == .visible {
                            HStack {
                                Spacer()
                                VStack {
                                    Spacer()
                                    Text("Overlay")
                                        .font(.largeTitle)
                                    Spacer()
                                }
                                Spacer()
                            }

                            Button {
                                self.onRestart()
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
                            .padding([.trailing, .top], 40)
                        }
                    }
                    .frame(maxWidth: /*@START_MENU_TOKEN@*/.infinity/*@END_MENU_TOKEN@*/, maxHeight: .infinity)
                }
                .ornament(attachmentAnchor: .scene(.bottom)) {
                    VStack {
                        // Add spacing between main window and ornament content to allow for the window resizer
                        Color.clear.frame(height: 200.0)

                        VStack {
                            HStack {
                                Text("Separation")

                                Slider(value: self.$separation, in: -5...5, step: 0.01, label: {
                                    Text("Separation")
                                }, minimumValueLabel: {
                                    Text("-5")
                                }) {
                                    Text("5")
                                }
                            }
                            Text("\(self.separation)")

                            HStack {
                                Text("3D overlay depth")

                                Slider(value: self.$depth, in: 0...10, step: 0.1, label: {
                                    Text("Depth")
                                }, minimumValueLabel: {
                                    Text("0")
                                }, maximumValueLabel: {
                                    Text("10")
                                })
                            }
                            Text("\(self.depth)")

                            Button("Toggle overlay") {
                                if self.controlVisibility == .visible {
                                    self.controlVisibility = .hidden
                                } else {
                                    self.controlVisibility = .visible
                                }
                            }
                        }
                        .safeAreaPadding()
                        .frame(width: 600)
                        .glassBackgroundEffect()
                    }
                }
                .onAppear {
                    self.emulator.separation = self.$separation
                    self.emulator.start()
                }
                .onDisappear {
                    self.emulator.stop()
                }
        }
    }
}
