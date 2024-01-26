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

let PIXEL_WIDTH = 384
let PIXEL_HEIGHT = 224
let PIXEL_COUNT = PIXEL_WIDTH * PIXEL_HEIGHT
let PIXEL_BYTE_COUNT = PIXEL_COUNT * 4

struct EmuView: View {
    let queue: DispatchQueue
    let virtualFriend: VirtualFriend!

    let context = CIContext()

    @StateObject var streamingStereoImage = StreamingStereoImage(image: StereoImage(left: nil, right: nil))

    @State var image: UIImage?

    init() {
        self.queue = DispatchQueue(label: "emu", qos: .userInteractive)

        let url = Bundle.main.url(forResource: "Mario's Tennis (Japan, USA)", withExtension: "vb")

        guard let url = url else {
            assertionFailure("Could not find embedded ROM")
            self.virtualFriend = nil
            return
        }

        self.virtualFriend = VirtualFriend(url.path(percentEncoded: false))
    }

    var body: some View {
        VStack {
            StreamingStereoImageView(width: 384, height: 224, stereoImage: self.streamingStereoImage)
                .onAppear(perform: {
                    self.queue.async {
                        while (true) {
                            let inputs = pollInput()

                            let frame = self.virtualFriend.run_frame(inputs)
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
                })

            if let image = image {
                Image(uiImage: image)
            }
        }
    }

    func pollInput() -> FFIGamepadInputs {
        let input = GCController.current?.input.capture()

        let a = input?.buttons[.a]?.pressedInput.isPressed ?? false
        let b = input?.buttons[.b]?.pressedInput.isPressed ?? false

        let rightTrigger = input?.buttons[.rightTrigger]?.pressedInput.isPressed ?? false
        let leftTrigger = input?.buttons[.leftTrigger]?.pressedInput.isPressed ?? false

        let sticks = GCController.current?.extendedGamepad?.capture()

        let rightDpadDown = sticks?.rightThumbstick.down.isPressed ?? false
        let rightDpadUp = sticks?.rightThumbstick.up.isPressed ?? false
        let rightDpadLeft = sticks?.rightThumbstick.left.isPressed ?? false
        let rightDpadRight = sticks?.rightThumbstick.right.isPressed ?? false

        let dpad = input?.dpads[.directionPad]
        let leftDpadRight = dpad?.right.isPressed ?? false
        let leftDpadLeft = dpad?.left.isPressed ?? false
        let leftDpadUp = dpad?.up.isPressed ?? false
        let leftDpadDown = dpad?.down.isPressed ?? false

//        let start = sticks?.buttonOptions?.isPressed ?? false
//        let select = input?.buttons[.options]?.pressedInput.isPressed ?? false
        // TODO: I can't get the proper options and menu buttons to work
        let start = input?.buttons[.y]?.pressedInput.isPressed ?? false
        let select = input?.buttons[.x]?.pressedInput.isPressed ?? false

        return FFIGamepadInputs(a_button: a, b_button: b, right_trigger: rightTrigger, left_trigger: leftTrigger, right_dpad_up: rightDpadUp, right_dpad_right: rightDpadRight, right_dpad_left: rightDpadLeft, right_dpad_down: rightDpadDown, left_dpad_up: leftDpadUp, left_dpad_right: leftDpadRight, left_dpad_left: leftDpadLeft, left_dpad_down: leftDpadDown, start: start, select: select)
    }
}

func rustVecToCIImage(_ vec: RustVec<UInt8>) -> CIImage {
    var bytes = [UInt8](repeating: 0, count: PIXEL_BYTE_COUNT)

    for i in 0..<PIXEL_COUNT {
        let value = vec[i]

        bytes[i * 4] = value
        bytes[i * 4 + 1] = 0
        bytes[i * 4 + 2] = 0
        // Alpha
        bytes[i * 4 + 3] = 255
    }

    let bitmapData = Data(bytes)

    return CIImage(bitmapData: bitmapData, bytesPerRow: PIXEL_WIDTH * 4, size: .init(width: PIXEL_WIDTH, height: PIXEL_HEIGHT), format: .RGBA8, colorSpace: CGColorSpace(name: CGColorSpace.sRGB)!)
}

#Preview {
    EmuView()
}
