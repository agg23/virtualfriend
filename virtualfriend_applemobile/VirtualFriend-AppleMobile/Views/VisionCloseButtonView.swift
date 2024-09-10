//
//  CloseButton.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 2/17/24.
//

import SwiftUI

struct VisionCloseButtonView: View {
    let action: () -> Void

    var body: some View {
        // Taken from https://old.reddit.com/r/SwiftUI/comments/okc2i9/what_is_the_best_way_to_achieve_this_xbutton_like/
        Button(role: .cancel, action: self.action, label: {
            Circle()
                .fill(Color(.secondarySystemBackground))
                .frame(width: 32, height: 32)
                .overlay(
                    Image(systemName: "xmark")
                        .font(.system(size: 16, weight: .bold, design: .rounded))
                        .foregroundColor(.secondary)
                )
        })
        .buttonBorderShape(.circle)
        .help("Close")
    }
}

#Preview {
    VisionCloseButtonView {

    }
}
