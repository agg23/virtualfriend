//
//  EmuContentView.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/9/24.
//

import SwiftUI

struct EmuContentView: View {
    @Environment(MainRouter.self) private var router
    @Environment(\.horizontalSizeClass) private var horizontalSizeClass

    @LEDBackgroundColor private var ledBackgroundColor
    @EyeSeparation private var separation

    @State private var controlVisibilityTimer: Timer?
    @State private var preventControlDismiss: Bool = false
    @State private var controlVisibility: Visibility = .hidden

    let controlsTimerDuration = 3.0

    let emulator: EmulatorStatus
    let controller: EmuController
    let title: String

    let onRestart: () -> Void

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

//        let toastText = Binding<ToastText> {
//            .content(text: self.controller.notification.text, icon: self.controller.notification.icon)
//        } set: { text in
//            if text == .none {
//                self.controller.notification = .none
//            }
//        }

        //        ToastWrapper(text: toastText) {
            ZStack(alignment: alignment) {
                // Background color to surround the view and pad out the window AR
                self.ledBackgroundColor
                    .ignoresSafeArea()

                Group {
                    switch self.emulator {
                    case .emulator(let emulator):
                        EmuImageView(emulator: emulator, controlVisibility: self.$controlVisibility, preventControlDismiss: self.$preventControlDismiss)
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
        .onChange(of: self.preventControlDismiss) { _, newValue in
            if newValue {
                self.clearTimer()
            } else {
                self.resetTimer()
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
                EmuHeaderOverlayView(title: self.title) {
                    self.resetTimer()
                } onBack: {
                    self.router.currentRoute = .main

                    if case .emulator(let emulator) = self.emulator {
                        emulator.shutdown()
                    }
                } onRestart: {
                    self.onRestart()
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

#Preview {
    EmuContentView(emulator: .none, controller: EmuController(), title: "Test Title", onRestart: {

    })
    .environment(MainRouter())
}
