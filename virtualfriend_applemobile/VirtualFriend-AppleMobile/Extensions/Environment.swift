//
//  Environment.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/7/24.
//

import SwiftUI

private struct ColorKey: EnvironmentKey {
    static let defaultValue: Color = .black
}

private struct TouchColorKey: EnvironmentKey {
    static let defaultValue: Color = .white
}

extension EnvironmentValues {
    var buttonColor: Color {
        get { self[ColorKey.self] }
        set { self[ColorKey.self] = newValue }
    }

    var touchColor: Color {
        get { self[TouchColorKey.self] }
        set { self[TouchColorKey.self] = newValue }
    }
}
