//
//  DpadView.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/6/24.
//

import SwiftUI

struct DpadView: View {
    @Environment(\.buttonColor) private var color

    @State private var size: CGSize = .zero

    let controller: TouchController
    let prefix: String?

    let onButtonChange: (_ direction: DpadDirection, _ pressed: Bool) -> Void

    var body: some View {
        let prefix = self.prefix ?? ""

        let upleft = "\(prefix)upleft"
        let up = "\(prefix)up"
        let upright = "\(prefix)upright"

        let left = "\(prefix)left"
        let right = "\(prefix)right"

        let downleft = "\(prefix)downleft"
        let down = "\(prefix)down"
        let downright = "\(prefix)downright"

        DpadLayout {
            // Row 1
            DpadCorner(controller: self.controller, name: upleft) { pressed in
                self.onButtonChange(.upLeft, pressed)
            }

            DpadArm(controller: self.controller, name: up, cornerMatches: [upleft, upright]) { pressed in
                self.onButtonChange(.up, pressed)
            }

            DpadCorner(controller: self.controller, name: upright) { pressed in
                self.onButtonChange(.upRight, pressed)
            }

            // Row 2
            DpadArm(controller: self.controller, name: left, cornerMatches: [upleft, downleft]) { pressed in
                self.onButtonChange(.left, pressed)
            }

            Rectangle()
                .fill(self.color)

            DpadArm(controller: self.controller, name: right, cornerMatches: [upright, downright]) { pressed in
                self.onButtonChange(.right, pressed)
            }

            // Row 3
            DpadCorner(controller: self.controller, name: downleft) { pressed in
                self.onButtonChange(.downLeft, pressed)
            }

            DpadArm(controller: self.controller, name: down, cornerMatches: [downleft, downright]) { pressed in
                self.onButtonChange(.down, pressed)
            }

            DpadCorner(controller: self.controller, name: downright) { pressed in
                self.onButtonChange(.downRight, pressed)
            }
        }
    }
}

private struct DpadArm: View {
    @Environment(\.buttonColor) private var color
    @Environment(\.touchColor) private var touchColor

    let controller: TouchController
    let name: String

    let cornerMatches: [String]

    let onButtonChange: (_ pressed: Bool) -> Void

    var body: some View {
        let isActive = self.controller.isActive(with: name) || self.cornerMatches.contains(where: { self.controller.isActive(with: $0) })

        Rectangle()
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
    }
}

private struct DpadCorner: View {
    let controller: TouchController
    let name: String

    let onButtonChange: (_ pressed: Bool) -> Void

    var body: some View {
        Color.clear
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
    }
}

struct DpadLayout: Layout {
    func sizeThatFits(proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) -> CGSize {
        let width = proposal.width ?? 100
        let height = proposal.height ?? 100

        let minDimension = min(width, height)

        return CGSize(width: minDimension, height: minDimension)
    }
    
    func placeSubviews(in bounds: CGRect, proposal: ProposedViewSize, subviews: Subviews, cache: inout ()) {
        let width = proposal.width ?? 100
        let height = proposal.height ?? 100

        let minDimension = min(width, height)

        let barThickness = minDimension * 0.25

        let barLength = (minDimension - barThickness) / 2

        for index in subviews.indices {
            let view = subviews[index]

            if index == 1 || index == 7 {
                // Top and bottom arms
                view.place(at: bounds.origin + CGPoint(x: barLength, y: index == 7 ? barLength + barThickness : 0), anchor: .topLeading, proposal: ProposedViewSize(width: barThickness, height: barLength))
            } else if index == 3 || index == 5 {
                // Left and right arms
                view.place(at: bounds.origin + CGPoint(x: index == 5 ? barLength + barThickness : 0, y: barLength), proposal: ProposedViewSize(width: barLength, height: barThickness))
            } else if index == 4 {
                view.place(at: bounds.origin + CGPoint(x: barLength, y: barLength), proposal: ProposedViewSize(width: barThickness, height: barThickness))
            } else {
                view.place(at: bounds.origin + CGPoint(x: index % 3 == 0 ? 0 : barLength + barThickness, y: index > 3 ? barLength + barThickness : 0), proposal: ProposedViewSize(width: barLength, height: barLength))
            }
        }
    }
}

enum DpadDirection {
    case up
    case down
    case left
    case right

    case upLeft
    case upRight
    case downLeft
    case downRight
}

#Preview {
    DpadView(controller: TouchController(), prefix: nil) { _, _ in }
}
