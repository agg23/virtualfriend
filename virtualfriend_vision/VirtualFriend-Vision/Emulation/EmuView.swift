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
                EmuContentView(emulator: emulator, controlVisibility: self.controlVisibility) {
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
    let controlVisibility: Visibility

    let onRestart: () -> Void

    var body: some View {
        StereoImageView(width: 384, height: 224, scale: 1.0, stereoImageChannel: self.emulator.stereoImageChannel)
            .overlay {
                ZStack(alignment: .topTrailing) {
                    // Clear does not get drawn on top of the StereoImageView for some reason
                    Color.white.opacity(0.0001)

                    if self.controlVisibility == .visible {
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
            .onAppear {
                self.emulator.start()
            }
            .onDisappear {
                self.emulator.stop()
            }
    }
}
