//
//  TriggerView.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/7/24.
//

import SwiftUI

struct TriggerView: View {
    let controller: TouchController

    let name: String
    let title: String
    let color: Color
    let width: CGFloat
    let height: CGFloat
    let onButtonChange: (_ pressed: Bool) -> Void

    var body: some View {
        Capsule()
            .stroke(.black)
            .background {
                GeometryReader { geometry in
                    let frame = geometry.frame(in: .named(self.controller.COORDINATE_SPACE_NAME))

                    Capsule().fill(self.color)
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
            }
            .frame(width: self.width, height: self.height)
    }
}

#Preview {
    TriggerView(controller: TouchController(), name: "l", title: "L", color: .red, width: 100, height: 20) { _ in }
}
