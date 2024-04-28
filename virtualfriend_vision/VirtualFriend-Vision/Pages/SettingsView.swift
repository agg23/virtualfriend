//
//  SettingsView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 4/27/24.
//

import SwiftUI

struct SettingsView: View {
    @AppStorage("romDirectoryBookmark") var romDirectoryBookmark: Data?

    @State var selectFolder = false

    var body: some View {
        NavigationStack {
            Form {
                //            Section("Navigation") {
                //                LanguageFilterPickerView(language: self.$filterLanguage, title: "Filter Language")
                //                Toggle(isOn: self.$hideMature) {
                //                    Text("Hide Mature Streams")
                //                    Text("Will not hide mature streams from streamers you follow")
                //                }
                //                Toggle(isOn: self.$disableIncrementingStreamDuration) {
                //                    Text("Disable Updating Stream Durations")
                //                    Text("Minimizes motion by displaying a static duration for a live stream")
                //                }
                //            }
                //
                //            Section("Playback") {
                //                Toggle("Shrink Video Corners", isOn: self.$smallBorderRadius)
                //                Toggle("Dim Surroundings", isOn: self.$dimSurroundings)
                //            }

                Section {
                    Button {
                        self.selectFolder.toggle()
                    } label: {
                        Text("Choose title directory")
                    }
                    .customFileImporter(self.$selectFolder, onOpen: { _, bookmark in
                        self.romDirectoryBookmark = bookmark

                        self.selectFolder = false
                    })
                }
            }
            .navigationTitle("Settings")
        }
    }
}

#Preview {
    SettingsView()
}
