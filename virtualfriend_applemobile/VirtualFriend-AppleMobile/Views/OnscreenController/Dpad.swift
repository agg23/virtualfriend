//
//  Dpad.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/6/24.
//

import SwiftUI

struct Dpad: View {
    @State private var size: CGSize = .zero

    let color: Color

    var body: some View {
        let barThickness = min(self.size.width * 0.25, self.size.height * 0.25)

        let widthLength = (self.size.width - barThickness) / 2
        let heightLength = (self.size.height - barThickness) / 2

        let barLength = min(widthLength, heightLength)

        ZStack {
            Color.clear
                .background {
                    GeometryReader { geometry in
                        Color.clear
                            .onChange(of: geometry.size, initial: true) { _, newValue in
                                self.size = newValue
                            }
                    }
                }

            // Grid will grow to only take the space it wants to, centered in this bounding box
            Grid {
                GridRow {
                    Spacer()

                    DpadArm(color: self.color, width: barThickness, height: barLength) { touched in
                        print("Up \(touched ? "on" : "off")")
                    }

                    Spacer()
                }

                GridRow {
                    DpadArm(color: self.color, width: barLength, height: barThickness) { touched in
                        print("Left \(touched ? "on" : "off")")
                    }

                    Spacer()

                    DpadArm(color: self.color, width: barLength, height: barThickness) { touched in
                        print("Right \(touched ? "on" : "off")")
                    }
                }
                .background {
                    // Xcode keeps throwing a fit if I replace the spacer above with the color, so we do this instead
                    self.color
                }

                GridRow {
                    Spacer()

                    DpadArm(color: self.color, width: barThickness, height: barLength) { touched in
                        print("Down \(touched ? "on" : "off")")
                    }

                    Spacer()
                }
            }
        }
    }
}

private struct DpadArm: View {
    @State private var state: TouchGestureStatus = .none

    let color: Color
    let width: CGFloat
    let height: CGFloat
    let onTouchChanged: (Bool) -> Void

    var body: some View {
        Rectangle()
            .fill(self.color)
            .frame(width: self.width, height: self.height)
            .touchGesture(state: self.$state, size: CGSize(width: self.width, height: self.height), onTouchChanged: self.onTouchChanged)
    }
}

#Preview {
    Dpad(color: .red)
        .frame(width: 100, height: 100)
}
