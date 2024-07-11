//
//  FaceButtonView.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/5/24.
//

import SwiftUI

struct FaceButtonView: View {
    @Environment(\.buttonColor) private var color
    @Environment(\.touchColor) private var touchColor

    @State private var size: CGSize = .zero

    let controller: TouchController

    let name: String
    let title: String

    let onButtonChange: (_ pressed: Bool) -> Void

    var body: some View {
        let isActive = self.controller.isActive(with: name)

        Circle()
            .fill(isActive ? self.touchColor : self.color)
            .background {
                GeometryReader { geometry in
                    let frame = geometry.frame(in: .named(self.controller.COORDINATE_SPACE_NAME))

                    Color.clear
                        .onDisappear {
                            self.controller.deregister(named: self.name)
                        }
                        .onChange(of: frame, initial: true, { _, newValue in
                            self.controller.register(named: self.name, frame: frame, callback: self.onButtonChange)
                        })
                        .onChange(of: geometry.size, initial: true) { _, newValue in
                            self.size = newValue
                        }
                }
            }
            .overlay {
                Text(self.title)
                    .fontWeight(.bold)
                    .foregroundStyle(.white)
            }
    }
}

#Preview {
    FaceButtonView(controller: TouchController(), name: "start", title: "Start") { _ in }
}
