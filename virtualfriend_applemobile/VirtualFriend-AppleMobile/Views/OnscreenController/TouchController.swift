//
//  TouchController.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/6/24.
//

import UIKit

@Observable class TouchController {
    @ObservationIgnored private var registeredButtons: [String: ButtonRegistration] = [:]

    var activeButtons: Set<String> = Set()

    var referenceView: UIView?

    let COORDINATE_SPACE_NAME = "onscreenController"

    func register(named: String, frame: CGRect, callback: @escaping (_ pressed: Bool) -> Void) {
        let isPressed = if let existingRegistration = self.registeredButtons[named] {
            existingRegistration.isPressed
        } else {
            false
        }

        self.registeredButtons[named] = ButtonRegistration(isPressed: isPressed, frame: frame, callback: callback)
    }

    func deregister(named: String) {
        self.registeredButtons[named] = nil
    }

    func update(touches: Set<UITouch>) {
        guard let view = self.referenceView else {
            return
        }

        for (name, registration) in self.registeredButtons {
            var isPressed = false

            for touch in touches {
                if registration.frame.contains(touch.location(in: view)) {
                    isPressed = true
                }
            }

            if registration.isPressed != isPressed {
                // Pressed changed
//                print("\(name) is \(isPressed)")
                self.registeredButtons[name] = ButtonRegistration(isPressed: isPressed, frame: registration.frame, callback: registration.callback)

                if isPressed {
                    self.activeButtons.insert(name)
                } else {
                    self.activeButtons.remove(name)
                }

                registration.callback(isPressed)
            }
        }
    }

    func isActive(with name: String) -> Bool {
        self.activeButtons.contains(name)
    }
}

private struct ButtonRegistration {
    var isPressed: Bool
    var frame: CGRect
    var callback: (_ pressed: Bool) -> Void
}
