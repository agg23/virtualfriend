//
//  TouchGesture.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/6/24.
//

import SwiftUI

private struct WithTouchGesture: ViewModifier {
    @Binding var state: TouchGestureStatus

    let size: CGSize
    let onTouchChanged: (Bool) -> Void

    func body(content: Content) -> some View {
        content
            .highPriorityGesture(
                DragGesture(minimumDistance: 0.0, coordinateSpace: .local)
                    .onChanged({ value in
                        guard self.state != .endedOutOfBounds else {
                            // If we're out of bounds, we're just waiting for gesture to properly call onEnded
                            return
                        }

                        if value.location.x >= 0.0 && value.location.x <= self.size.width && value.location.y >= 0.0 && value.location.y <= self.size.height {
                            if self.state == .none {
                                self.onTouchChanged(true)
                            }

                            self.state = .started
                        } else {
                            if self.state == .started {
                                self.onTouchChanged(false)
                            }

                            self.state = .endedOutOfBounds
                        }
                    })
                    .onEnded({ value in
                        if self.state != .endedOutOfBounds {
                            // We need to send final ended touch event
                            self.onTouchChanged(false)
                        }

                        self.state = .none
                    })
            )
    }
}

extension View {
    func touchGesture(state: Binding<TouchGestureStatus>, size: CGSize, onTouchChanged: @escaping (Bool) -> Void) -> some View {
        modifier(WithTouchGesture(state: state, size: size, onTouchChanged: onTouchChanged))
    }
}

enum TouchGestureStatus {
    case started
    case endedOutOfBounds
    case none
}
