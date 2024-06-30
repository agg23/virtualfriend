//
//  Settings.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 5/4/24.
//

import SwiftUI

struct Settings {
    static let ledBackgroundColorKey = "ledBackgroundColor"
    static let ledForegroundColorKey = "ledForegroundColor"

    static let eyeSeparationKey = "eyeSeparation"

    static func parseStoredColor(for key: String) -> Color? {
        guard let string = UserDefaults.standard.string(forKey: key) else {
            return nil
        }

        let components = string.split(separator: "_")

        guard components.count == 3,
              let red = Double(components[0]),
              let green = Double(components[1]),
              let blue = Double(components[2]) else {
            // Incorrect number of segments or couldn't parse components
            UserDefaults.standard.setValue(nil, forKey: key)
            return nil
        }

        return Color(red: red, green: green, blue: blue)
    }

    static func storeColor(for key: String, color: Color) {
        let components = color.resolve(in: .init()).cgColor.components!

        UserDefaults.standard.setValue("\(components[0])_\(components[1])_\(components[2])", forKey: key)
    }
}

@propertyWrapper
struct LEDBackgroundColor: DynamicProperty {
    @AppStorage(Settings.ledBackgroundColorKey) var wrappedValue: Color = .init(red: 0.0, green: 0.0, blue: 0.0)

    var projectedValue: Binding<Color> {
        self._wrappedValue.projectedValue
    }
}

@propertyWrapper
struct LEDForegroundColor: DynamicProperty {
    @AppStorage(Settings.ledForegroundColorKey) var wrappedValue: Color = .init(red: 1.0, green: 0.0, blue: 0.0)

    var projectedValue: Binding<Color> {
        self._wrappedValue.projectedValue
    }
}

class LEDColorWrapper: ObservableObject {
    @AppStorage(Settings.ledForegroundColorKey) var foreground: Color = .init(red: 1.0, green: 0.0, blue: 0.0)
    @AppStorage(Settings.ledBackgroundColorKey) var background: Color = .init(red: 0.0, green: 0.0, blue: 0.0)

    var cache: VBColor = VBColor(foregroundColor: .init(red: 1.0, green: 0.0, blue: 0.0, alpha: 1.0), backgroundColor: .init(red: 0.0, green: 0.0, blue: 0.0, alpha: 1.0))
}

@propertyWrapper
struct LEDColor: DynamicProperty {
    @StateObject var colorWrapper: LEDColorWrapper = LEDColorWrapper()

    var wrappedValue: VBColor {
        self.colorWrapper.cache
    }

    func update() {
        self.colorWrapper.cache = VBColor(foregroundColor: self.colorWrapper.foreground.rawCGColor, backgroundColor: self.colorWrapper.background.rawCGColor)
    }
}

@propertyWrapper
struct EyeSeparation: DynamicProperty {
    @AppStorage(Settings.eyeSeparationKey) var wrappedValue: Double = 0.0

    var projectedValue: Binding<Double> {
        self._wrappedValue.projectedValue
    }
}
