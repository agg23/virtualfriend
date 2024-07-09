//
//  EmuControllerView.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/7/24.
//

import SwiftUI

struct EmuControllerView: View {
    let controller: EmuController

    var body: some View {
        ZStack(alignment: .bottom) {
            // Fill screen
            Color.clear

            ControllerView() { direction, pressed in
                switch direction {
                case .up:
                    self.controller.leftDpadUp = pressed
                case .down:
                    self.controller.leftDpadDown = pressed
                case .left:
                    self.controller.leftDpadLeft = pressed
                case .right:
                    self.controller.leftDpadRight = pressed

                case .upLeft:
                    self.controller.leftDpadUpLeft = pressed
                case .upRight:
                    self.controller.leftDpadUpRight = pressed
                case .downLeft:
                    self.controller.leftDpadDownLeft = pressed
                case .downRight:
                    self.controller.leftDpadDownRight = pressed
                }
            } rightDpad: { direction, pressed in
                switch direction {
                case .up:
                    self.controller.rightDpadUp = pressed
                case .down:
                    self.controller.rightDpadDown = pressed
                case .left:
                    self.controller.rightDpadLeft = pressed
                case .right:
                    self.controller.rightDpadRight = pressed

                case .upLeft:
                    self.controller.rightDpadUpLeft = pressed
                case .upRight:
                    self.controller.rightDpadUpRight = pressed
                case .downLeft:
                    self.controller.rightDpadDownLeft = pressed
                case .downRight:
                    self.controller.rightDpadDownRight = pressed
                }
            } aButton: { pressed in
                self.controller.aButton = pressed
            } bButton: { pressed in
                self.controller.bButton = pressed
            } startButton: { pressed in
                self.controller.startButton = pressed
            } selectButton: { pressed in
                self.controller.selectButton = pressed
            } lButton: { pressed in
                self.controller.lButton = pressed
            } rButton: { pressed in
                self.controller.rButton = pressed
            }
        }
    }
}

#Preview {
    EmuControllerView(controller: EmuController())
}
