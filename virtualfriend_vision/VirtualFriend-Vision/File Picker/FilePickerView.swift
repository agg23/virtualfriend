//
//  FilePickerView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/26/24.
//

import SwiftUI
import RealityKit

struct FilePickerView: View {
    @State var toggle: Bool

    init() {
        self.toggle = false
    }

    var body: some View {
        Grid {
            ForEach(0..<3) { _ in
                GridRow {
                    FilePickerEntry()
                    FilePickerEntry()
                    FilePickerEntry()
                }
            }
        }

            VStack (spacing: 12) {
                Text("Test")
                Toggle(isOn: $toggle, label: {
                    Text("Toggle")
                })
            }
            .frame(width: 600)
            .padding(36)
            .glassBackgroundEffect()
    }
}

#Preview {
    FilePickerView()
}
