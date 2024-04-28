//
//  VirtualFriend_VisionApp.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/9/24.
//

import SwiftUI

@main
struct VirtualFriend_VisionApp: App {
    init() {
        // Print simulator run location
        print(NSHomeDirectory())
    }

    var body: some Scene {
        WindowGroup(id: "filepicker") {
            MainWindowView()
                .frame(minWidth: 600, minHeight: 400)
        }
        .windowResizability(.contentSize)
        // Default window size
        .defaultSize(width: 1280, height: 720)

        WindowGroup(id: "emu", for: URL.self) { url in
            if let url = url.wrappedValue {
                EmuView(fileUrl: url)
                    .windowGeometryPreferences(minimumSize: CGSize(width: 384 + 2, height: 224 + 2), resizingRestrictions: .uniform)
            } else {
                Text("Could not start emulator")
            }
        }
        .windowResizability(.contentSize)
        .windowStyle(.plain)
        .defaultSize(width: 2, height: 2, depth: 0.1, in: .meters)
    }
}
