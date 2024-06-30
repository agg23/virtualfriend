//
//  MainWindowView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 4/27/24.
//

import SwiftUI

struct MainWindowView: View {
    @Environment(\.dismissWindow) private var dismissWindow

    @State private var router = MainRouter()

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
                .onAppear {
                    if self.id == nil {
                        // This window shouldn't exist
                        dismissWindow()
                    }
                }
            case .emulator(let url):
                EmuView(fileUrl: url)
            }
        }
        .environment(self.router)
        #if os(visionOS)
        #else
        .tint(.red)
        #endif
    }
}

#Preview {
    MainWindowView(id: "foo")
}
