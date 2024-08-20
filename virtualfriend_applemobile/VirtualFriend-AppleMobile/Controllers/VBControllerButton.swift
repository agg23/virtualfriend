//
//  VBControllerButton.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 8/18/24.
//

import Foundation

enum VBControllerButton: Identifiable {
    var id: Self { self }

    case a
    case b

    case start
    case select

    case leftTrigger
    case rightTrigger

    case rightDpadUp
    case rightDpadDown
    case rightDpadLeft
    case rightDpadRight

    case leftDpadUp
    case leftDpadDown
    case leftDpadLeft
    case leftDpadRight

    func name() -> String {
        switch self {
        case .a:
            return "A Button"
        case .b:
            return "B Button"
        case .start:
            return "Start"
        case .select:
            return "Select"
        case .leftTrigger:
            return "Left Trigger"
        case .rightTrigger:
            return "Right Trigger"
        case .rightDpadUp:
            fallthrough
        case .leftDpadUp:
            return "Up"
        case .rightDpadDown:
            fallthrough
        case .leftDpadDown:
            return "Down"
        case .rightDpadLeft:
            fallthrough
        case .leftDpadLeft:
            return "Left"
        case .rightDpadRight:
            fallthrough
        case .leftDpadRight:
            return "Right"
        }
    }

    func icon() -> String {
        switch self {
        case .a:
            return "a.circle.fill"
        case .b:
            return "b.circle.fill"
        case .start:
            // TODO: Set from print statement in Emulator
            break
        case .select:
            // TODO: Set from print statement in Emulator
            break
        case .leftTrigger:
            return "l.button.roundedbottom.horizontal.fill"
        case .rightTrigger:
            return "r.button.roundedbottom.horizontal.fill"
        case .rightDpadUp:
            return "circle.grid.cross.up.filled"
        case .rightDpadDown:
            return "circle.grid.cross.down.filled"
        case .rightDpadLeft:
            return "circle.grid.cross.left.filled"
        case .rightDpadRight:
            return "circle.grid.cross.right.filled"
        case .leftDpadUp:
            return "circle.grid.cross.up.filled"
        case .leftDpadDown:
            return "circle.grid.cross.down.filled"
        case .leftDpadLeft:
            return "circle.grid.cross.left.filled"
        case .leftDpadRight:
            return "circle.grid.cross.right.filled"
        }

        return "questionmark.folder"
    }
}
