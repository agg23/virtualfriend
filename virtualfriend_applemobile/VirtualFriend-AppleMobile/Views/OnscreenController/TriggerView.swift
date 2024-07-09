//
//  TriggerView.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/7/24.
//

import SwiftUI

struct TriggerView: View {
    @Environment(\.buttonColor) private var color
    @Environment(\.touchColor) private var touchColor

    let controller: TouchController

    let name: String
    let title: String
    let width: CGFloat
    let height: CGFloat

    let onButtonChange: (_ pressed: Bool) -> Void

    var body: some View {
        let isActive = self.controller.isActive(with: name)

        Capsule()
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
                }
            }
            .overlay {
                Text(self.title)
                    .fontWeight(.bold)
                    .foregroundStyle(.white)
            }
            .frame(width: self.width, height: self.height)
    }
}

#Preview {
    TriggerView(controller: TouchController(), name: "l", title: "L", width: 100, height: 20) { _ in }
        .environment(\.buttonColor, .red)
}
