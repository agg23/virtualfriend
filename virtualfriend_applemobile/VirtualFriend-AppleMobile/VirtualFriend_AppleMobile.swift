//
//  VirtualFriend_AppleMobile.swift
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

        // Create Titles directory
        var titlesUrl = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
        titlesUrl.append(component: "Titles")

        // Create Saves directory
        var savesUrl = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
        savesUrl.append(component: "Saves")

        // Create Savestates directory
        var savestatesUrl = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
        savestatesUrl.append(component: "Savestates")

        // withIntermediateDirectories allows the call to succeed even if there's no directory created
        do {
            try FileManager.default.createDirectory(at: titlesUrl, withIntermediateDirectories: true)
            try FileManager.default.createDirectory(at: savesUrl, withIntermediateDirectories: true)
            try FileManager.default.createDirectory(at: savestatesUrl, withIntermediateDirectories: true)
        } catch {
            print("Could not create titles/saves directory \(error)")
        }
    }

    var body: some Scene {
        // We use a data type (for:) so we can reopen/focus the same window
        WindowGroup(for: String?.self) { $id in
            // This `nil` ID is used to bypass a bug. See inside of `MainWindowView`
            MainWindowView(id: id)
                #if os(visionOS)
                .frame(minWidth: 600, minHeight: 400)
                #endif
        } defaultValue: {
            // Set default value so there's a shared ID we can use to reuse the window
            // TODO: This doesn't work for some reason
            return "main" as String?
        }
        #if os(visionOS)
        // Window set to plain so the ImmersiveView doesn't show any window borders
        .windowStyle(.plain)
        // Macs crash with contentSize
        .windowResizability(.contentSize)
        // Default window size
        .defaultSize(width: 1280, height: 720)
        #endif

        #if os(visionOS)
        ImmersiveSpace(id: "ImmersiveSpace") {
            ImmersiveView()
        }
        .immersionStyle(selection: Binding(get: {
            .progressive
        }, set: { _ in }), in: .progressive)
        #endif
    }
}
