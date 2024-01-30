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

struct EmuView: View {
    let queue = DispatchQueue(label: "emu", qos: .userInteractive)

    let context = CIContext()

    @Binding var fileUrl: URL?

    @StateObject var streamingStereoImage = StreamingStereoImage(image: StereoImage(left: nil, right: nil))
    @State var virtualFriend: VirtualFriend?

    @State var image: UIImage?

    init(fileUrl: Binding<URL?>) {
        self._fileUrl = fileUrl

        self.virtualFriend = VirtualFriend(fileUrl.wrappedValue!.path(percentEncoded: false))
    }

    var body: some View {
        VStack {
            StreamingStereoImageView(width: 384, height: 224, stereoImage: self.streamingStereoImage)
                .onAppear(perform: {
                    self.queue.async {
                        while (true) {
                            autoreleasepool {
                                let inputs = pollInput()

                                guard let frame = self.virtualFriend?.run_frame(inputs) else {
                                    return
                                }
                                let leftImage = rustVecToCIImage(frame.left)
                                let rightImage = rustVecToCIImage(frame.right)

                                DispatchQueue.main.async {
                                    self.image = UIImage(cgImage: context.createCGImage(leftImage, from: leftImage.extent)!)
                                }

                                // TODO: This should be flipped by Metal, not the CPU
                                let leftTransformedImage = leftImage.transformed(by: .init(scaleX: 1, y: -1))
                                let rightTransformedImage = rightImage.transformed(by: .init(scaleX: 1, y: -1))

                                DispatchQueue.main.async {
                                    self.streamingStereoImage.update(left: leftTransformedImage, right: rightTransformedImage)
                                }
                            }
                        }
                    }
                })

            if let image = image {
                Image(uiImage: image)
            }
        }
        .onChange(of: self.fileUrl) {
            self.virtualFriend = VirtualFriend(self.fileUrl!.path(percentEncoded: false))
        }
        .onAppear {
            // I'm not sure why init doesn't cover this
            self.virtualFriend = VirtualFriend(self.fileUrl!.path(percentEncoded: false))
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

#Preview {
    EmuView(fileUrl: .constant(nil))
}
