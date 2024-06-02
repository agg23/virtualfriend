//
//  MainWindowView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 4/27/24.
//

import SwiftUI

struct MainWindowView: View {
    @Environment(\.dismissWindow) private var dismissWindow

    /// Hack to prevent crash when saved stream windows are opened after reboot with a `nil` id
    /// Present in 1.2
    let id: String?

    var body: some View {
        TabView {
            FilePickerView()
                .tabItem {
                    Label("Titles", systemImage: "rectangle.stack")
                }

            SettingsView()
                .tabItem {
                    Label("Settings", systemImage: "gear")
                }
        }
        .onAppear {
            if self.id == nil {
                // This window shouldn't exist
                dismissWindow()
            }
        }
    }
}

#Preview {
    MainWindowView(id: "foo")
}
