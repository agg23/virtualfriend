//
//  FFIGamepadInputs.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/25/24.
//

import Foundation

extension FFIGamepadInputs {
    func merge(_ input_b: FFIGamepadInputs) -> FFIGamepadInputs {
        let a = self.a_button || input_b.a_button
        let b = self.b_button || input_b.b_button

        let leftTrigger = self.left_trigger || input_b.left_trigger
        let rightTrigger = self.right_trigger || input_b.right_trigger

        let rightDpadDown = self.right_dpad_down || input_b.right_dpad_down
        let rightDpadUp = self.right_dpad_up || input_b.right_dpad_up
        let rightDpadLeft = self.right_dpad_left || input_b.right_dpad_left
        let rightDpadRight = self.right_dpad_right || input_b.right_dpad_right

        let leftDpadDown = self.left_dpad_down || input_b.left_dpad_down
        let leftDpadUp = self.left_dpad_up || input_b.left_dpad_up
        let leftDpadLeft = self.left_dpad_left || input_b.left_dpad_left
        let leftDpadRight = self.left_dpad_right || input_b.left_dpad_right

        let start = self.start || input_b.start
        let select = self.select || input_b.select

        return FFIGamepadInputs(a_button: a, b_button: b, right_trigger: rightTrigger, left_trigger: leftTrigger, right_dpad_up: rightDpadUp, right_dpad_right: rightDpadRight, right_dpad_left: rightDpadLeft, right_dpad_down: rightDpadDown, left_dpad_up: leftDpadUp, left_dpad_right: leftDpadRight, left_dpad_left: leftDpadLeft, left_dpad_down: leftDpadDown, start: start, select: select)
    }
}
