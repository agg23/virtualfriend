//
//  Color.swift
//  VirtualFriend-Vision
//
//  Created by Eric Lewis on 5/4/24.
//
// MIT License
//Copyright (c) 2024 Eric Lewis
//
//Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
//
//The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
//
//THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.


import SwiftUI

extension Color {
    var rawCGColor: CGColor {
        get {
            self.resolve(in: .init()).cgColor
        }
        set {
            
        }
    }
}

extension Color: RawRepresentable {
    public typealias RawValue = String

    public init?(rawValue: String) {
        guard let data = Data(base64Encoded: rawValue),
              let color = try? NSKeyedUnarchiver.unarchivedObject(ofClass: UIColor.self, from: data) else {
            return nil
        }
        self = Color(color)
    }

    public var rawValue: String {
        let data = try? NSKeyedArchiver.archivedData(
            withRootObject: UIColor(self), requiringSecureCoding: false)
        return data?.base64EncodedString() ?? ""
    }
}

extension Color {
    // Taken from https://stackoverflow.com/questions/596216/formula-to-determine-perceived-brightness-of-rgb-color
    var isDark: Bool {
        var r, g, b, a: CGFloat
        r = 0.0
        g = 0.0
        b = 0.0
        a = 0.0
        UIColor(self).getRed(&r, green: &g, blue: &b, alpha: &a)
        return  (0.2126 * r + 0.7152 * g + 0.0722 * b) < 0.50
    }
}
