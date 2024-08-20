//
//  Controller.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 8/13/24.
//

import Foundation

let faceVBButtons: [VBControllerButton] = [.a, .b, .start, .select]
let triggerVBButtons: [VBControllerButton] = [.leftTrigger, .rightTrigger]
let leftDpadVBButtons: [VBControllerButton] = [.leftDpadUp, .leftDpadDown, .leftDpadLeft, .leftDpadRight]
let rightDpadVBButtons: [VBControllerButton] = [.rightDpadUp, .rightDpadDown, .rightDpadLeft, .rightDpadRight]

let faceGCButtons: [GCControllerButton] = [.a, .b, .x, .y]
let triggerGCButtons: [GCControllerButton] = [.leftTrigger, .rightTrigger, .leftBumper, .rightBumper]
let menuGCButtons: [GCControllerButton] = [.menu, .options]
let leftThumbstickGCButtons: [GCControllerButton] = [.leftThumbstick(.up), .leftThumbstick(.down), .leftThumbstick(.left), .leftThumbstick(.right)]
let rightThumbstickGCButtons: [GCControllerButton] = [.rightThumbstick(.up), .rightThumbstick(.down), .rightThumbstick(.left), .rightThumbstick(.right)]
let dpadGCButtons: [GCControllerButton] = [.dpad(.up), .dpad(.down), .dpad(.left), .dpad(.right)]
