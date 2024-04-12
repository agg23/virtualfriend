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
            FilePickerView()
        }
        .windowResizability(.contentSize)
        // Use depth to restrict the size of RealityView. Not restricting it will result in the
        // window displaying back by an additional `depth`
        .defaultSize(width: 1, height: 1, depth: 0.1, in: .meters)
        .defaultSize(width: 1400, height: 1100)

        WindowGroup(id: "emu", for: URL.self) { url in
            if let url = url.wrappedValue {
                EmuView(fileUrl: url)
            } else {
                Text("Could not start emulator")
            }
        }
//        WindowGroup(id: "emu") {
//            let url = Bundle.main.url(forResource: "BLOX 2", withExtension: "vb")!
//            EmuView(fileUrl: .constant(url))
//        }
        .windowStyle(.volumetric)
        .defaultSize(width: 2, height: 2, depth: 0.1, in: .meters)
    }
}
