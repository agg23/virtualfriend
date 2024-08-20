//
//  GCControllerButton.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 8/18/24.
//

import GameController

enum GCControllerButton: Identifiable, Hashable {
    var id: String {
        self.baseName()
    }

    case a
    case b
    case x
    case y

    case leftTrigger
    case rightTrigger
    case leftBumper
    case rightBumper

    case menu
    case options

    case leftThumbstick(GCControllerDirection)
    case rightThumbstick(GCControllerDirection)
    case dpad(GCControllerDirection)

    func name() -> String {
        return self.gcName() ?? self.baseName()
    }

    private func gcName() -> String? {
        let existingController = GCController.current?.input
        let existingExtendedController = GCController.current?.extendedGamepad

        switch (self) {
        case .a:
            return existingController?.buttons[.a]?.localizedName
        case .b:
            return existingController?.buttons[.a]?.localizedName
        case .x:
            return existingController?.buttons[.x]?.localizedName
        case .y:
            return existingController?.buttons[.y]?.localizedName

        case .menu:
            return existingController?.buttons[.menu]?.localizedName
        case .options:
            return existingController?.buttons[.options]?.localizedName

        case .leftTrigger:
            return existingController?.buttons[.leftTrigger]?.localizedName
        case .rightTrigger:
            return existingController?.buttons[.rightTrigger]?.localizedName
        case .leftBumper:
            return existingController?.buttons[.leftShoulder]?.localizedName
        case .rightBumper:
            return existingController?.buttons[.rightShoulder]?.localizedName

        case .leftThumbstick(let direction):
            switch (direction) {
            case .up:
                return existingExtendedController?.leftThumbstick.up.localizedName
            case .down:
                return existingExtendedController?.leftThumbstick.down.localizedName
            case .left:
                return existingExtendedController?.leftThumbstick.left.localizedName
            case .right:
                return existingExtendedController?.leftThumbstick.right.localizedName
            }

        case .rightThumbstick(let direction):
            switch (direction) {
            case .up:
                return existingExtendedController?.rightThumbstick.up.localizedName
            case .down:
                return existingExtendedController?.rightThumbstick.down.localizedName
            case .left:
                return existingExtendedController?.rightThumbstick.left.localizedName
            case .right:
                return existingExtendedController?.rightThumbstick.right.localizedName
            }

        case .dpad(let direction):
            switch (direction) {
            case .up:
                return existingExtendedController?.dpad.up.localizedName
            case .down:
                return existingExtendedController?.dpad.down.localizedName
            case .left:
                return existingExtendedController?.dpad.left.localizedName
            case .right:
                return existingExtendedController?.dpad.right.localizedName
            }
        }
    }

    private func baseName() -> String {
        switch (self) {
        case .a:
            return "A"
        case .b:
            return "B"
        case .x:
            return "X"
        case .y:
            return "Y"

        case .menu:
            return "Menu"
        case .options:
            return "Options"

        case .leftTrigger:
            return "Left Trigger"
        case .rightTrigger:
            return "Right Trigger"
        case .leftBumper:
            return "Left Bumper"
        case .rightBumper:
            return "Right Bumper"

        case .leftThumbstick(let direction):
            return direction.name()
        case .rightThumbstick(let direction):
            return direction.name()
        case .dpad(let direction):
            return direction.name()
        }
    }

    func icon() -> String {
        let existingController = GCController.current?.input

        switch self {
        case .a:
            return existingController?.buttons[.a]?.sfSymbolsName ?? "a.circle.fill"
        case .b:
            return existingController?.buttons[.b]?.sfSymbolsName ?? "b.circle.fill"
        case .x:
            return existingController?.buttons[.x]?.sfSymbolsName ?? "x.circle.fill"
        case .y:
            return existingController?.buttons[.y]?.sfSymbolsName ?? "y.circle.fill"

        case .menu:
            // TODO: Set from print statement in Emulator
            return existingController?.buttons[.menu]?.sfSymbolsName ?? "foo"
        case .options:
            // TODO: Set from print statement in Emulator
            return existingController?.buttons[.options]?.sfSymbolsName ?? "foo"

        case .leftTrigger:
            return existingController?.buttons[.leftTrigger]?.sfSymbolsName ?? "lt.button.roundedtop.horizontal.fill"
        case .leftBumper:
            return existingController?.buttons[.leftShoulder]?.sfSymbolsName ?? "l2.button.roundedtop.horizontal.fill"
        case .rightTrigger:
            return existingController?.buttons[.rightTrigger]?.sfSymbolsName ??  "rt.button.roundedtop.horizontal.fill"
        case .rightBumper:
            return existingController?.buttons[.rightShoulder]?.sfSymbolsName ?? "r2.button.roundedtop.horizontal.fill"

        case .leftThumbstick(let direction):
            switch (direction) {
            case .up:
                return "l.joystick.tilt.up.fill"
            case .down:
                return "l.joystick.tilt.down.fill"
            case .left:
                return "l.joystick.tilt.left.fill"
            case .right:
                return "l.joystick.tilt.right.fill"
            }
        case .rightThumbstick(let direction):
            switch (direction) {
            case .up:
                return "r.joystick.tilt.up.fill"
            case .down:
                return "r.joystick.tilt.down.fill"
            case .left:
                return "r.joystick.tilt.left.fill"
            case .right:
                return "r.joystick.tilt.right.fill"
            }
        case .dpad(let direction):
            switch (direction) {
            case .up:
                return "circle.grid.cross.up.filled"
            case .down:
                return "circle.grid.cross.down.filled"
            case .left:
                return "circle.grid.cross.left.filled"
            case .right:
                return "circle.grid.cross.right.filled"
            }
        }
    }

}

enum GCControllerDirection: Identifiable {
    var id: Self { self }

    case up
    case down
    case left
    case right

    func name() -> String {
        switch (self) {
        case .up:
            "Up"
        case .down:
            "Down"
        case .left:
            "Left"
        case .right:
            "Right"
        }
    }
}
