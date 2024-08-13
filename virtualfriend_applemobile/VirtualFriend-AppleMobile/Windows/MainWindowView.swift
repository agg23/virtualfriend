//
//  MainWindowView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 4/27/24.
//

import SwiftUI
import GameController

struct MainWindowView: View {
    #if os(visionOS)
    @Environment(\.openImmersiveSpace) private var openImmersiveSpace
    @Environment(\.dismissImmersiveSpace) private var dismissImmersiveSpace
    #endif
    @Environment(\.dismissWindow) private var dismissWindow

    @State private var router = MainRouter()
    @State private var immersiveModel = ImmersiveModel()

    /// Hack to prevent crash when saved stream windows are opened after reboot with a `nil` id
    /// Present in 1.2
    let id: String?

    var body: some View {
        Group {
            switch router.currentRoute {
            case .main:
                TabView {
                    FilePickerView()
                        .tabItem {
                            Label("Library", systemImage: Icon.library)
                        }

                    SettingsView()
                        .tabItem {
                            Label("Settings", systemImage: Icon.settings)
                        }
                }
                #if os(visionOS)
                .glassBackgroundEffect()
                #endif
                .onAppear {
                    if self.id == nil {
                        // This window shouldn't exist
                        dismissWindow()
                    }
                }
            case .emulator(let entry):
                EmuView(fileEntry: entry)
            }
        }
        #if !os(visionOS)
        .tint(.red)
        #endif
        .environment(self.router)
        .environment(self.immersiveModel)
        .mount({
            // Prefetch controllers so we don't see a controller notification even though we didn't just connect a controller
            print("Startup with \(GCController.controllers().count) controllers")

            #if os(visionOS)
            self.immersiveModel.initialize(openAction: self.openImmersiveSpace, dismissAction: self.dismissImmersiveSpace)
            #endif
        }, unmount: {
            #if os(visionOS)
            Task {
                self.immersiveModel.isImmersed = false
                await self.dismissImmersiveSpace()
            }
            #endif
        })
        .onAppear {
            // Prefetch controllers so we don't see a controller notification even though we didn't just connect a controller
            print("Startup with \(GCController.controllers().count) controllers")
        }
    }
}

#Preview {
    MainWindowView(id: "foo")
}
