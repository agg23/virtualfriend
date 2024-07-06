//
//  FaceButton.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/5/24.
//

import SwiftUI

struct FaceButton: View {
    @State private var size: CGSize = .zero
    @State private var state: TouchGestureStatus = .none

    let title: String
    let color: Color
    let touchColor: Color

    let onTouchChanged: (Bool) -> Void

    var body: some View {
        let isPressed = self.state == .started

        Circle()
            .stroke(.black)
            .background {
                GeometryReader { geometry in
                    Circle().fill(self.color)
                        .onChange(of: geometry.size, initial: true) { _, newValue in
                            self.size = newValue
                        }
                }
            }
            .overlay {
                let pressedSize = self.size.width * 0.9

                Circle()
                    .fill(self.touchColor)
                    .frame(width: isPressed ? pressedSize : nil, height: isPressed ? pressedSize : nil)
                    .animation(.linear(duration: 0.10), value: isPressed)
            }
            .overlay {
                Text(self.title)
            }
            .touchGesture(state: self.$state, size: self.size, onTouchChanged: self.onTouchChanged)
    }
}

#Preview {
    FaceButton(title: "Start", color: .blue, touchColor: .init(red: 0, green: 0, blue: 0.9)) { touched in
        print("Touch \(touched ? "on" : "off")")
    }
}
