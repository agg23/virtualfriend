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
        WindowGroup {
            ContentView()
        }.windowStyle(.volumetric)

        ImmersiveSpace(id: "ImmersiveSpace") {
            ImmersiveView()
        }.immersionStyle(selection: .constant(.progressive), in: .progressive)
    }
}
