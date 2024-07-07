//
//  FaceButtonView.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/5/24.
//

import SwiftUI

struct FaceButtonView: View {
    @State private var size: CGSize = .zero

    let controller: TouchController

    let name: String
    let title: String
    let color: Color
    let touchColor: Color

    var body: some View {
        let isPressed = false

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
            .background {
                GeometryReader { geometry in
                    let frame = geometry.frame(in: .named(self.controller.COORDINATE_SPACE_NAME))

                    Color.clear
                        .onDisappear {
                            self.controller.deregister(named: self.name)
                        }
                        .onChange(of: frame, initial: true, { _, newValue in
                            self.controller.register(named: self.name, frame: frame)
                        })
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
    }
}

#Preview {
    FaceButtonView(controller: TouchController(), name: "start", title: "Start", color: .blue, touchColor: .init(red: 0, green: 0, blue: 0.9))
}
