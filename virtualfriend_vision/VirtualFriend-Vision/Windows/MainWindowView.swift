//
//  MainWindowView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 4/27/24.
//

import SwiftUI

struct MainWindowView: View {
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
    }
}

#Preview {
    MainWindowView()
}
