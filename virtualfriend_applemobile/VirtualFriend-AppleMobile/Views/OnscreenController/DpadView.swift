//
//  DpadView.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/6/24.
//

import SwiftUI

struct DpadView: View {
    @Environment(\.buttonColor) private var color

    let controller: TouchController
    let prefix: String?
    let width: CGFloat
    let height: CGFloat

    let onButtonChange: (_ direction: DpadDirection, _ pressed: Bool) -> Void

    var body: some View {
        let barThickness = min(self.width * 0.25, self.height * 0.25)

        let widthLength = (self.width - barThickness) / 2
        let heightLength = (self.height - barThickness) / 2

        let barLength = min(widthLength, heightLength)

        let prefix = self.prefix ?? ""

        let upleft = "\(prefix)upleft"
        let up = "\(prefix)up"
        let upright = "\(prefix)upright"

        let left = "\(prefix)left"
        let right = "\(prefix)right"

        let downleft = "\(prefix)downleft"
        let down = "\(prefix)down"
        let downright = "\(prefix)downright"

        Grid(horizontalSpacing: 0, verticalSpacing: 0) {
            GridRow {
                DpadCorner(controller: self.controller, name: upleft, width: barLength, height: barLength) { pressed in
                    self.onButtonChange(.upLeft, pressed)
                }

                DpadArm(controller: self.controller, name: up, cornerMatches: [upleft, upright], width: barThickness, height: barLength) { pressed in
                    self.onButtonChange(.up, pressed)
                }

                DpadCorner(controller: self.controller, name: upright, width: barLength, height: barLength) { pressed in
                    self.onButtonChange(.upRight, pressed)
                }
            }

            GridRow {
                DpadArm(controller: self.controller, name: left, cornerMatches: [upleft, downleft], width: barLength, height: barThickness) { pressed in
                    self.onButtonChange(.left, pressed)
                }

                Rectangle()
                    .fill(self.color)

                DpadArm(controller: self.controller, name: right, cornerMatches: [upright, downright], width: barLength, height: barThickness) { pressed in
                    self.onButtonChange(.right, pressed)
                }
            }

            GridRow {
                DpadCorner(controller: self.controller, name: downleft, width: barLength, height: barLength) { pressed in
                    self.onButtonChange(.downLeft, pressed)
                }

                DpadArm(controller: self.controller, name: down, cornerMatches: [downleft, downright], width: barThickness, height: barLength) { pressed in
                    self.onButtonChange(.down, pressed)
                }

                DpadCorner(controller: self.controller, name: downright, width: barLength, height: barLength) { pressed in
                    self.onButtonChange(.downRight, pressed)
                }
            }
        }
        .frame(width: self.width, height: self.height)
    }
}

private struct DpadArm: View {
    @Environment(\.buttonColor) private var color
    @Environment(\.touchColor) private var touchColor

    let controller: TouchController
    let name: String
    let cornerMatches: [String]
    let width: CGFloat
    let height: CGFloat

    let onButtonChange: (_ pressed: Bool) -> Void

    var body: some View {
        let isActive = self.controller.isActive(with: name) || self.cornerMatches.contains(where: { self.controller.isActive(with: $0) })

        Rectangle()
            .fill(isActive ? self.touchColor : self.color)
            .frame(width: self.width, height: self.height)
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
    let width: CGFloat
    let height: CGFloat

    let onButtonChange: (_ pressed: Bool) -> Void

    var body: some View {
        Color.clear
            .frame(width: self.width, height: self.height)
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
    DpadView(controller: TouchController(), prefix: nil, width: 100, height: 100) { _, _ in }
}
