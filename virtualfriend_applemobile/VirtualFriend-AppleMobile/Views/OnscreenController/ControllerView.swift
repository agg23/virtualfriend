//
//  ControllerView.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/6/24.
//

import SwiftUI

struct ControllerView: View {
    @LEDBackgroundColor private var ledBackgroundColor

    @State private var leftController: TouchController = TouchController()
    @State private var rightController: TouchController = TouchController()

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
            ControllerSideView(controller: self.leftController, triggerName: "l", triggleTitle: "L", triggerOnButtonChange: self.lButton, dpadPrefix: "left", dpadOnButtonChange: self.leftDpad, leftButtonName: "select", leftButtonTitle: "Select", leftButtonOnButtonChange: self.selectButton, rightButtonName: "start", rightButtonTitle: "Start", rightButtonOnButtonChange: self.startButton)
                .padding([.leading, .top, .bottom], 24)
                .overlay {
                    TouchGestureView(controller: self.leftController)
                }

            Spacer()

            ControllerSideView(controller: self.rightController, triggerName: "r", triggleTitle: "R", triggerOnButtonChange: self.rButton, dpadPrefix: "right", dpadOnButtonChange: self.rightDpad, leftButtonName: "b", leftButtonTitle: "B", leftButtonOnButtonChange: self.bButton, rightButtonName: "a", rightButtonTitle: "A", rightButtonOnButtonChange: self.aButton)
                .padding([.trailing, .top, .bottom], 24)
                .overlay {
                    TouchGestureView(controller: self.rightController)
                }
        }
        .overlay {
            DummyCoordinateView(leftController: self.leftController, rightController: self.rightController)
                .allowsHitTesting(false)
        }
        // Declare area shared by TouchGestureViews
        .coordinateSpace(.named(self.leftController.COORDINATE_SPACE_NAME))
        .environment(\.buttonColor, self.ledBackgroundColor.isDark ? .init(white: 0.4, opacity: 0.5) : .init(white: 0.6, opacity: 0.5))
        .environment(\.touchColor, self.ledBackgroundColor.isDark ? .init(white: 0.6, opacity: 0.5) : .init(white: 0.8, opacity: 0.5))
    }
}

/// Used solely for establishing the view reference from for shared touch coordinate checking
struct DummyCoordinateView: UIViewRepresentable {
    typealias UIViewType = UIView

    let leftController: TouchController
    let rightController: TouchController

    func makeUIView(context: Context) -> UIView {
        let view = UIView(frame: .zero)

        self.leftController.referenceView = view
        self.rightController.referenceView = view

        return view
    }

    func updateUIView(_ uiView: UIView, context: Context) {

    }
}

private struct ControllerSideView: View {
    @State private var size: CGSize = .zero

    let controller: TouchController

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
        ControllerLayout {
            TriggerView(controller: self.controller, name: self.triggerName, title: self.triggleTitle, onButtonChange: self.triggerOnButtonChange)

            DpadView(controller: self.controller, prefix: self.dpadPrefix, onButtonChange: self.dpadOnButtonChange)
                .padding(.vertical, 16)

            HStack {
                FaceButtonView(controller: self.controller, name: self.leftButtonName, title: self.leftButtonTitle, onButtonChange: self.leftButtonOnButtonChange)

                Spacer()

                FaceButtonView(controller: self.controller, name: self.rightButtonName, title: self.rightButtonTitle, onButtonChange: self.rightButtonOnButtonChange)
            }
        }
        .background {
            GeometryReader { geometry in
                Color.clear
                    .onChange(of: geometry.size, initial: true) { _, newValue in
                        self.size = newValue
                    }
            }
        }
        .frame(maxWidth: 200)
    }
}

private struct ControllerLayout: Layout {
    private let TRIGGER_PERCENTAGE = 0.13
    private let DPAD_PERCENTAGE = 0.6
    private let BUTTON_PERCENTAGE = 0.27

    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let width = proposal.width ?? 100
        let height = proposal.height ?? 100

        let calculatedHeight = self.calcuateViewHeight(proposedHeight: height, width: width)

        return CGSize(width: width, height: calculatedHeight)
    }
    
    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        let width = proposal.width ?? 100
        let height = proposal.height ?? 100

        var offsetY = 0.0

        for index in subviews.indices {
            let view = subviews[index]

            switch index {
            case 0:
                view.place(at: bounds.origin, proposal: ProposedViewSize(width: width, height: height * self.TRIGGER_PERCENTAGE))

                offsetY += height * self.TRIGGER_PERCENTAGE
            case 1:
                let actualHeight = self.dpadHeight(systemHeight: height, width: width)

                let proposedSize = ProposedViewSize(width: width, height: actualHeight)

                let dimensions = view.dimensions(in: proposedSize)

                let offsetX = (width - dimensions.width) / 2

                view.place(at: bounds.origin + CGPoint(x: offsetX, y: offsetY), proposal: proposedSize)

                offsetY += actualHeight
            default:
                view.place(at: bounds.origin + CGPoint(x: 0, y: offsetY), proposal: ProposedViewSize(width: width, height: height * self.BUTTON_PERCENTAGE))
            }
        }
    }

    private func dpadHeight(systemHeight: CGFloat, width: CGFloat) -> CGFloat {
        let expectedHeight = systemHeight * DPAD_PERCENTAGE
        return min(width, expectedHeight)
    }

    private func calcuateViewHeight(proposedHeight: CGFloat, width: CGFloat) -> CGFloat {
        // Use knowledge of dpad size calculation to determine the proposed height of the entire view
        let proposedDpadHeight = self.dpadHeight(systemHeight: proposedHeight, width: width)

        return proposedDpadHeight / DPAD_PERCENTAGE
    }
}

#Preview {
    ControllerView { _, _ in
        
    } rightDpad: { _, _ in

    } aButton: { _ in

    } bButton: { _ in

    } startButton: { _ in

    } selectButton: { _ in

    } lButton: { _ in

    } rButton: { _ in

    }
}
