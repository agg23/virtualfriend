//
//  VirtualFriend_VisionApp.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/9/24.
//

import SwiftUI

@main
struct VirtualFriend_VisionApp: App {
    @Environment(\.openWindow) private var openWindow;

    var body: some Scene {
        WindowGroup(id: "filepicker") {
//            FilePickerView()
//                .onAppear {
//                    print("Appear")
//                    openWindow(id: "test")
//                }
        }
        .windowStyle(.volumetric)
//        .defaultSize(width: 1, height: 1, depth: 0.1, in: .meters)

        WindowGroup(id: "test") {
//            ContentView()
            FilePickerView()
            Text("test")
            Button(action: {
                openWindow(id: "filepicker")
            }) {
                Text("Open")
            }
//            EmuView()
        }
//        .defaultSize(CGSize(width: 1000, height: 1000))
        .defaultSize(width: 1, height: 1, depth: 0.1, in: .meters)
    }
}
