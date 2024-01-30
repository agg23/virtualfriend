//
//  VirtualFriend_VisionApp.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/9/24.
//

import SwiftUI

@main
struct VirtualFriend_VisionApp: App {
    var body: some Scene {
        WindowGroup(id: "filepicker") {
            FilePickerView()
        }
        .windowResizability(.contentSize)
        // Use depth to restrict the size of RealityView. Not restricting it will result in the
        // window displaying back by an additional `depth`
        .defaultSize(width: 1, height: 1, depth: 0.1, in: .meters)
        .defaultSize(width: 1400, height: 1100)

        WindowGroup(id: "emu", for: URL.self) { url in
            EmuView(fileUrl: url)
        }
        .windowStyle(.volumetric)
    }
}
