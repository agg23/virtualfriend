//
//  ControllerView.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/6/24.
//

import SwiftUI

struct ControllerView: View {
    @State private var controller: TouchController = TouchController()
    @State private var size: CGSize = .zero

    let leftDpad: (_ direction: DpadDirection, _ pressed: Bool) -> Void
    let rightDpad: (_ direction: DpadDirection, _ pressed: Bool) -> Void

    let aButton: (_ pressed: Bool) -> Void
    let bButton: (_ pressed: Bool) -> Void

    let startButton: (_ pressed: Bool) -> Void
    let selectButton: (_ pressed: Bool) -> Void

    let lButton: (_ pressed: Bool) -> Void
    let rButton: (_ pressed: Bool) -> Void

    var body: some View {
        let sideWidth = self.size.width * 0.4

        HStack {
            ControllerSideView(controller: self.controller, width: sideWidth, triggerName: "l", triggleTitle: "L", triggerOnButtonChange: self.lButton, dpadPrefix: "left", dpadOnButtonChange: self.leftDpad, leftButtonName: "select", leftButtonTitle: "Sel", leftButtonOnButtonChange: self.selectButton, rightButtonName: "start", rightButtonTitle: "Start", rightButtonOnButtonChange: self.startButton)
                .padding([.leading, .top, .bottom], 24)

            Spacer()

            ControllerSideView(controller: self.controller, width: sideWidth, triggerName: "r", triggleTitle: "R", triggerOnButtonChange: self.rButton, dpadPrefix: "right", dpadOnButtonChange: self.rightDpad, leftButtonName: "b", leftButtonTitle: "B", leftButtonOnButtonChange: self.bButton, rightButtonName: "a", rightButtonTitle: "A", rightButtonOnButtonChange: self.aButton)
                .padding([.trailing, .top, .bottom], 24)
        }
        // Use full width as available touch area
        .frame(minWidth: 0, maxWidth: .infinity)
        .background {
            GeometryReader { geometry in
                Color.clear
                    .onChange(of: geometry.size, initial: true) { _, newValue in
                        self.size = newValue
                    }
            }
        }
        .overlay {
            TouchGestureView(controller: self.controller)
        }
        .coordinateSpace(.named(self.controller.COORDINATE_SPACE_NAME))
        .environment(\.buttonColor, .init(white: 0.2, opacity: 0.5))
        .environment(\.touchColor, .init(white: 0.4, opacity: 0.5))
    }
}

private struct ControllerSideView: View {
    let controller: TouchController

    let width: CGFloat

    let triggerName: String
    let triggleTitle: String
    let triggerOnButtonChange: (_ pressed: Bool) -> Void

    let dpadPrefix: String
    let dpadOnButtonChange: (_ direction: DpadDirection, _ pressed: Bool) -> Void

    let leftButtonName: String
    let leftButtonTitle: String
    let leftButtonOnButtonChange: (_ pressed: Bool) -> Void

    let rightButtonName: String
    let rightButtonTitle: String
    let rightButtonOnButtonChange: (_ pressed: Bool) -> Void

    var body: some View {
        VStack {
            TriggerView(controller: self.controller, name: self.triggerName, title: self.triggleTitle, width: self.width, height: 30, onButtonChange: self.triggerOnButtonChange)

            DpadView(controller: self.controller, prefix: self.dpadPrefix, width: self.width, height: self.width, onButtonChange: self.dpadOnButtonChange)
                .padding(.vertical, 16)

            HStack {
                FaceButtonView(controller: self.controller, name: self.leftButtonName, title: self.leftButtonTitle, onButtonChange: self.leftButtonOnButtonChange)

                Spacer(minLength: self.width * 0.2)

                FaceButtonView(controller: self.controller, name: self.rightButtonName, title: self.rightButtonTitle, onButtonChange: self.rightButtonOnButtonChange)
            }
            .frame(width: self.width)
        }
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
