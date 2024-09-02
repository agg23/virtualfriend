//
//  EmuContentView.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/9/24.
//

import SwiftUI

struct EmuContentView: View {
    @Environment(MainRouter.self) private var router
    @Environment(ImmersiveModel.self) private var immersiveModel
    @Environment(\.horizontalSizeClass) private var horizontalSizeClass

    @LEDBackgroundColor private var ledBackgroundColor
    @EyeSeparation private var separation
    @Enable3D private var enable3D

    @State private var controlVisibilityTimer: Timer?
    @State private var controlVisibility: Visibility = .hidden

    @State private var showSavestates: Bool = false

    let controlsTimerDuration = 3.0
    let verticalBaseImagePadding = 16.0
    let visionOSImagePadding = 8.0

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
                if !self.immersiveModel.isImmersed {
                    // Only display background color if not over the immersed black background
                    self.ledBackgroundColor
                        .ignoresSafeArea()
                }

                Group {
                    switch self.emulator {
                    case .emulator(let emulator):
                        // Only autostart if we're not in the overlay (i.e. pressed restart)
                        EmuImageView(emulator: emulator, autostart: self.controlVisibility == .hidden)
                            .padding(.vertical, self.verticalBaseImagePadding)
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
                .padding(self.visionOSImagePadding)
                #endif
            }
        //        }
        .onTapGesture {
            self.toggleVisibility()
        }
        #if !os(visionOS)
        .overlay {
            if self.controller.notification == .noController {
                EmuControllerView(controller: self.controller)
            }
        }
        #endif
        .overlay {
            if self.controlVisibility == .visible {
                Group {
                    self.ledBackgroundColor.isDark ? Color(white: 0.1, opacity: 0.7) : Color(white: 0.9, opacity: 0.7)
                }
                .ignoresSafeArea()
                .onTapGesture {
                    self.toggleVisibility()
                }
            }
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
        .sheet(isPresented: self.$showSavestates, onDismiss: {
            // Refresh overlay timer
//            self.resetTimer()
        }, content: {
            SavestatesView()
        })
    }

    @ViewBuilder
    var controlsOverlay: some View {
        ZStack {
            // Clear does not get drawn on top of the StereoImageView in visionOS for some reason
            Color.white.opacity(0.0001)
                .allowsHitTesting(false)

            if self.controlVisibility == .visible {
                let overlay = EmuHeaderOverlayView(title: self.title, isImmersed: self.immersiveModel.isImmersed) {
//                    self.resetTimer()
                } onBack: {
                    self.router.currentRoute = .main

                    if case .emulator(let emulator) = self.emulator {
                        emulator.shutdown()
                    }
                } onImmersive: {
                    Task {
                        if self.immersiveModel.isImmersed {
                            await self.immersiveModel.dismiss()
                        } else {
                            await self.immersiveModel.open()
                        }
                    }
                } onRestart: {
                    self.onRestart()
                } onCreateSavestate: {
                    // TODO
                } onOpenSavestates: {
                    self.showSavestates = true;
                }

                if self.immersiveModel.isImmersed {
                    // Add wrapping overlay padding as if other padding wasn't removed above
                    overlay
                        .padding(.vertical, self.verticalBaseImagePadding)
                        .padding(self.visionOSImagePadding)
                } else {
                    overlay
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

    func toggleVisibility() {
        if case .emulator(let emulator) = self.emulator {
            if self.controlVisibility == .visible {
                emulator.start()
            } else {
                emulator.shutdown()
            }
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
