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
//            EmuView()
        }
//        .defaultSize(CGSize(width: 1000, height: 1000))
        // Use depth to restrict the size of RealityView. Not restricting it will result in the
        // window displaying back by an additional `depth`
        .defaultSize(width: 1, height: 1, depth: 0.1, in: .meters)
        .defaultSize(width: 1400, height: 1100)
    }
}
