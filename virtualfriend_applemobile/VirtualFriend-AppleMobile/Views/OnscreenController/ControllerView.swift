//
//  ControllerView.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/6/24.
//

import SwiftUI

struct ControllerView: View {
    @State private var controller: TouchController = TouchController()

    let leftDpad: (_ direction: DpadDirection, _ pressed: Bool) -> Void
    let rightDpad: (_ direction: DpadDirection, _ pressed: Bool) -> Void

    let aButton: (_ pressed: Bool) -> Void
    let bButton: (_ pressed: Bool) -> Void

    let startButton: (_ pressed: Bool) -> Void
    let selectButton: (_ pressed: Bool) -> Void

    let lButton: (_ pressed: Bool) -> Void
    let rButton: (_ pressed: Bool) -> Void

    var body: some View {
        HStack {
            VStack {
                TriggerView(controller: self.controller, name: "l", title: "L", color: .red, width: 100, height: 30, onButtonChange: self.lButton)

                DpadView(controller: self.controller, prefix: "left", color: .red, width: 100, height: 100, onButtonChange: self.leftDpad)

                HStack {
                    FaceButtonView(controller: self.controller, name: "select", title: "Sel", color: .blue, touchColor: .red, onButtonChange: self.selectButton)

                    Spacer()

                    FaceButtonView(controller: self.controller, name: "start", title: "Start", color: .blue, touchColor: .red, onButtonChange: self.startButton)
                }
                .frame(width: 100)
            }
            .padding()

            Spacer()

            VStack {
                TriggerView(controller: self.controller, name: "r", title: "R", color: .red, width: 100, height: 30, onButtonChange: self.rButton)

                DpadView(controller: self.controller, prefix: "right", color: .red, width: 100, height: 100, onButtonChange: self.leftDpad)

                HStack {
                    FaceButtonView(controller: self.controller, name: "b", title: "B", color: .blue, touchColor: .red, onButtonChange: self.bButton)

                    Spacer()

                    FaceButtonView(controller: self.controller, name: "a", title: "A", color: .blue, touchColor: .red, onButtonChange: self.aButton)
                }
                .frame(width: 100)
            }
            .padding()

        }
        // Use full width as available touch area
        .frame(minWidth: 0, maxWidth: .infinity)
        .overlay {
            TouchGestureView(controller: self.controller)
        }
        .coordinateSpace(.named(self.controller.COORDINATE_SPACE_NAME))
    }
}

#Preview {
    ControllerView() { _, _ in
        
    } rightDpad: { _, _ in

    } aButton: { _ in

    } bButton: { _ in

    } startButton: { _ in

    } selectButton: { _ in

    } lButton: { _ in

    } rButton: { _ in

    }
}
