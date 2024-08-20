//
//  ControllerSettingsView.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 8/13/24.
//

import SwiftUI

struct ControllerSettingsView: View {
    var body: some View {
        Form {
            VBControllerButtonSection(name: "Face Buttons", buttons: faceVBButtons)

            VBControllerButtonSection(name: "Trigger Buttons", buttons: triggerVBButtons)

            VBControllerButtonSection(name: "Left D-pad", buttons: leftDpadVBButtons)
            VBControllerButtonSection(name: "Right D-pad", buttons: rightDpadVBButtons)
        }
    }
}

private struct VBControllerButtonSection: View {
    let name: String
    let buttons: [VBControllerButton]

    var body: some View {
        Section(self.name) {
            ForEach(self.buttons) { button in
                GCControllerButtonPicker(selection: .constant(.dpad(.up)), button: button)
            }
        }
    }
}

private struct GCControllerButtonPicker: View {
    @Binding var selection: GCControllerButton

    let button: VBControllerButton

    var body: some View {
        Picker(selection: self.$selection) {
            GCControllerButtonPickerSection(sectionName: "Face Buttons", sectionButtons: faceGCButtons)

            GCControllerButtonPickerSection(sectionName: "Menu Buttons", sectionButtons: menuGCButtons)

            GCControllerButtonPickerSection(sectionName: "Trigger Buttons", sectionButtons: triggerGCButtons)

            GCControllerButtonPickerSection(sectionName: "D-pad", sectionButtons: dpadGCButtons)

            GCControllerButtonPickerSection(sectionName: "Left Stick", sectionButtons: leftThumbstickGCButtons)
            GCControllerButtonPickerSection(sectionName: "Right Stick", sectionButtons: rightThumbstickGCButtons)
        } label: {
            Label(button.name(), systemImage: button.icon())
        }
    }
}

private struct GCControllerButtonPickerSection: View {
    let sectionName: String
    let sectionButtons: [GCControllerButton]

    var body: some View {
        Section(self.sectionName) {
            ForEach(self.sectionButtons) { button in
                Button(button.name(), systemImage: button.icon()) {

                }
                .id(button)
                .tag(button)
            }
        }
    }
}

#Preview {
    ControllerSettingsView()
}
