//
//  EmuView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/21/24.
//

import SwiftUI
import RealityKit
import VBStereoRenderRealityKit
import Combine

import GameController

class EmuObject {
    var virtualFriend: VirtualFriend

    var stereoImageStream: AsyncStream<StereoImage>!
    var continuation: AsyncStream<StereoImage>.Continuation!

    init?(fileUrl: URL) {
        let _ = fileUrl.startAccessingSecurityScopedResource()

        let data: Data

        do {
            data = try Data(contentsOf: fileUrl)
        } catch {
            print(error)
            return nil
        }

        let array = data.withUnsafeBytes { (pointer: UnsafeRawBufferPointer) -> [UInt8] in
            let buffer = pointer.bindMemory(to: UInt8.self)
            return buffer.map { UInt8($0) }
        }

        self.virtualFriend = array.withUnsafeBufferPointer { pointer in
            return VirtualFriend(pointer)
        }

        fileUrl.stopAccessingSecurityScopedResource()

        self.stereoImageStream = AsyncStream<StereoImage>(bufferingPolicy: .bufferingNewest(1)) { continuation in
            self.continuation = continuation
        }
    }
}

struct EmuView: View {
    let fileUrl: URL

    @State var emu: EmuObject?

    var body: some View {
        Group {
            if let emu = self.emu {
                EmuContentView(emu: emu)
            } else {
                Text("Could not start emulator")
            }
        }
        .onChange(of: self.fileUrl, initial: true) { _, newValue in
            self.emu = EmuObject(fileUrl: newValue)
        }
    }
}

private struct EmuContentView: View {
    let queue = DispatchQueue(label: "emu", qos: .userInteractive)

    let context = CIContext()

    let emu: EmuObject

    var body: some View {
        StereoImageView(width: 384, height: 224, scale: 1.0, stereoImageStream: self.emu.stereoImageStream)
            .task {
                self.queue.async {
                    while (true) {
                        autoreleasepool {
                            let inputs = pollInput()

                            let frame = self.emu.virtualFriend.run_frame(inputs)

                            let leftImage = rustVecToCIImage(frame.left)
                            let rightImage = rustVecToCIImage(frame.right)

                            // TODO: This should be flipped by Metal, not the CPU
                            let leftTransformedImage = leftImage.transformed(by: .init(scaleX: 1, y: -1))
                            let rightTransformedImage = rightImage.transformed(by: .init(scaleX: 1, y: -1))

                            self.emu.continuation.yield(StereoImage(left: leftTransformedImage, right: rightTransformedImage))
                        }
                    }
                }
            }
    }

    func pollInput() -> FFIGamepadInputs {
        let keyboard = pollKeyboardInput()
        let controller = pollControllerInput()

        return keyboard.merge(controller)
    }

    func pollKeyboardInput() -> FFIGamepadInputs {
        let keyboard = GCKeyboard.coalesced?.keyboardInput

        let a = keyboard?.button(forKeyCode: .spacebar)?.isPressed ?? false
        let b = keyboard?.button(forKeyCode: .keyC)?.isPressed ?? false

        let leftTrigger = keyboard?.button(forKeyCode: .one)?.isPressed ?? false
        let rightTrigger = keyboard?.button(forKeyCode: .three)?.isPressed ?? false

        let rightDpadDown = keyboard?.button(forKeyCode: .keyK)?.isPressed ?? false
        let rightDpadUp = keyboard?.button(forKeyCode: .keyI)?.isPressed ?? false
        let rightDpadLeft = keyboard?.button(forKeyCode: .keyJ)?.isPressed ?? false
        let rightDpadRight = keyboard?.button(forKeyCode: .keyL)?.isPressed ?? false

        let leftDpadDown = keyboard?.button(forKeyCode: .keyS)?.isPressed ?? false
        let leftDpadUp = keyboard?.button(forKeyCode: .keyW)?.isPressed ?? false
        let leftDpadLeft = keyboard?.button(forKeyCode: .keyA)?.isPressed ?? false
        let leftDpadRight = keyboard?.button(forKeyCode: .keyD)?.isPressed ?? false

        let start = keyboard?.button(forKeyCode: .keyQ)?.isPressed ?? false
        let select = keyboard?.button(forKeyCode: .keyE)?.isPressed ?? false

        return FFIGamepadInputs(a_button: a, b_button: b, right_trigger: rightTrigger, left_trigger: leftTrigger, right_dpad_up: rightDpadUp, right_dpad_right: rightDpadRight, right_dpad_left: rightDpadLeft, right_dpad_down: rightDpadDown, left_dpad_up: leftDpadUp, left_dpad_right: leftDpadRight, left_dpad_left: leftDpadLeft, left_dpad_down: leftDpadDown, start: start, select: select)
    }

    func pollControllerInput() -> FFIGamepadInputs {
        let input = GCController.current?.input.capture()

        let a = input?.buttons[.a]?.pressedInput.isPressed ?? false
        let b = input?.buttons[.b]?.pressedInput.isPressed ?? false

        let rightTrigger = input?.buttons[.rightTrigger]?.pressedInput.isPressed ?? false
        let leftTrigger = input?.buttons[.leftTrigger]?.pressedInput.isPressed ?? false

        let sticks = GCController.current?.extendedGamepad?.capture()

        // TODO: These controls are broken in the simulator due to https://developer.apple.com/forums/thread/734774
        let rightDpadDown = sticks?.rightThumbstick.down.isPressed ?? false
        let rightDpadUp = sticks?.rightThumbstick.up.isPressed ?? false
        let rightDpadLeft = sticks?.rightThumbstick.left.isPressed ?? false
        let rightDpadRight = sticks?.rightThumbstick.right.isPressed ?? false

        let leftDpadRight = sticks?.dpad.right.isPressed ?? false
        let leftDpadLeft = sticks?.dpad.left.isPressed ?? false
        let leftDpadUp = sticks?.dpad.up.isPressed ?? false
        let leftDpadDown = sticks?.dpad.down.isPressed ?? false

        let start = sticks?.buttonMenu.isPressed ?? false
        let select = sticks?.buttonOptions?.isPressed ?? false

        return FFIGamepadInputs(a_button: a, b_button: b, right_trigger: rightTrigger, left_trigger: leftTrigger, right_dpad_up: rightDpadUp, right_dpad_right: rightDpadRight, right_dpad_left: rightDpadLeft, right_dpad_down: rightDpadDown, left_dpad_up: leftDpadUp, left_dpad_right: leftDpadRight, left_dpad_left: leftDpadLeft, left_dpad_down: leftDpadDown, start: start, select: select)
    }
}
