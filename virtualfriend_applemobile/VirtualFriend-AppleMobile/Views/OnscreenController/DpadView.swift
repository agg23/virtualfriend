//
//  DpadView.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/6/24.
//

import SwiftUI

struct DpadView: View {
    @State private var size: CGSize = .zero

    let controller: TouchController
    let prefix: String?
    let color: Color
    let width: CGFloat
    let height: CGFloat

    let onButtonChange: (_ direction: DpadDirection, _ pressed: Bool) -> Void

    var body: some View {
        let barThickness = min(self.width * 0.25, self.height * 0.25)

        let widthLength = (self.width - barThickness) / 2
        let heightLength = (self.height - barThickness) / 2

        let barLength = min(widthLength, heightLength)

        let prefix = self.prefix ?? ""

        Grid(horizontalSpacing: 0, verticalSpacing: 0) {
            GridRow {
                DpadCorner(controller: self.controller, name: "\(prefix)upleft", width: barLength, height: barLength) { pressed in
                    self.onButtonChange(.upLeft, pressed)
                }

                DpadArm(controller: self.controller, name: "\(prefix)up", color: self.color, width: barThickness, height: barLength) { pressed in
                    self.onButtonChange(.up, pressed)
                }

                DpadCorner(controller: self.controller, name: "\(prefix)upright", width: barLength, height: barLength) { pressed in
                    self.onButtonChange(.upRight, pressed)
                }
            }

            GridRow {
                DpadArm(controller: self.controller, name: "\(prefix)left", color: self.color, width: barLength, height: barThickness) { pressed in
                    self.onButtonChange(.left, pressed)
                }

                Spacer()

                DpadArm(controller: self.controller, name: "\(prefix)right", color: self.color, width: barLength, height: barThickness) { pressed in
                    self.onButtonChange(.right, pressed)
                }
            }
            .background {
                // Xcode keeps throwing a fit if I replace the spacer above with the color, so we do this instead
                self.color
            }

            GridRow {
                DpadCorner(controller: self.controller, name: "\(prefix)downleft", width: barLength, height: barLength) { pressed in
                    self.onButtonChange(.downLeft, pressed)
                }

                DpadArm(controller: self.controller, name: "\(prefix)down", color: self.color, width: barThickness, height: barLength) { pressed in
                    self.onButtonChange(.down, pressed)
                }

                DpadCorner(controller: self.controller, name: "\(prefix)downright", width: barLength, height: barLength) { pressed in
                    self.onButtonChange(.downRight, pressed)
                }
            }
        }
        .frame(width: self.width, height: self.height)
//        .background {
//            GeometryReader { geometry in
//                let _ = print(geometry.size)
//
//                Color.clear
//                    .onChange(of: geometry.size, initial: true) { _, newValue in
//                        self.size = newValue
//                    }
//            }
//            .frame(width: 100, height: 100)
//        }
    }
}

private struct DpadArm: View {
    let controller: TouchController
    let name: String
    let color: Color
    let width: CGFloat
    let height: CGFloat

    let onButtonChange: (_ pressed: Bool) -> Void

    var body: some View {
        Rectangle()
            .fill(self.color)
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
    DpadView(controller: TouchController(), prefix: nil, color: .red, width: 100, height: 100) { _, _ in }
}
