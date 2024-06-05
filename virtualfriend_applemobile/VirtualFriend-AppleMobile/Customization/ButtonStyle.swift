//
//  ButtonStyle.swift
//  VirtualFriend-AppleMobile
//
//  Created by Adam Gastineau on 6/5/24.
//

import SwiftUI

struct BlackTextButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .foregroundColor(.primary) // This ensures it uses the primary color
    }
}
