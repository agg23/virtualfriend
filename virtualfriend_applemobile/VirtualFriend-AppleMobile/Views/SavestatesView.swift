//
//  SavestatesView.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 9/1/24.
//

import SwiftUI

struct SavestatesView: View {
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationView {
            Text("Hi")
                .toolbar {
                    ToolbarItem(placement: .cancellationAction) {
                        Button("Cancel", role: .cancel) {
                            dismiss()
                        }
                    }
                }
        }
    }
}

#Preview {
    SavestatesView()
}
