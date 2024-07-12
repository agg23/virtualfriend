//
//  KeyboardSettingsView.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/11/24.
//

import SwiftUI

struct KeyboardSettingsView: View {
    var body: some View {
        Form {
            Section("Left D-pad") {
                    KeybindView(control: "Up", key: "W")
                    KeybindView(control: "Left", key: "A")
                    KeybindView(control: "Down", key: "S")
                    KeybindView(control: "Right", key: "D")
            }

            Section("Left Face Buttons") {
                KeybindView(control: "Start", key: "X")

                KeybindView(control: "Select", key: "Z")
            }

            Section("Right D-pad") {
                KeybindView(control: "Up", key: "I")
                KeybindView(control: "Left", key: "J")
                KeybindView(control: "Down", key: "K")
                KeybindView(control: "Right", key: "L")
            }

            Section("Right Face Buttons") {
                KeybindView(control: "A", key: "Spacebar")

                KeybindView(control: "B", key: "M")
            }

            Section("Triggers") {
                KeybindView(control: "Left Trigger", key: "Q")

                KeybindView(control: "Right Trigger", key: "O")
            }
        }
        .navigationTitle("Keyboard")
    }
}

private struct KeybindView: View {
    let control: String
    let key: String

    var body: some View {
        HStack {
            Text(self.control)

            Spacer()

            Text(self.key)
        }
    }
}

#Preview {
    NavigationStack {
        KeyboardSettingsView()
    }
}
