//
//  Emulator.swift
//  VirtualFriend-AppleMobile
//
//  Created by Adam Gastineau on 6/29/24.
//

import GameController

private let SAMPLE_RATE = 41667
private let FRAME_RATE = 50.0
private let AUDIO_FRAMES_PER_LOOP: UInt = 400

class Emulator {
    private let fileName: String

    private var emulatorQueue: DispatchQueue

    private let virtualFriend: VirtualFriend
    private let audio: EmulatorAudio

    private var prevFrameTime: TimeInterval?

    private var halt = true

    var stereoImageChannel = AsyncImageChannel()

    // Settings
    var enableSound: Bool {
        get {
            self.audio.enableSound
        }
        set {
            self.audio.enableSound = newValue
        }
    }

    var color: VBColor
    var separation: Double = 0.0

    init(fileUrl: URL) throws {
        self.fileName = String(fileUrl.lastPathComponent.split(separator: ".")[0])

        self.emulatorQueue = DispatchQueue(label: "emulator", qos: .userInteractive)

        let didAccessScope = fileUrl.startAccessingSecurityScopedResource()

        let data: Data

        do {
            data = try Data(contentsOf: fileUrl)
        } catch {
            print(didAccessScope, error)
            throw error
        }

        let array = data.withUnsafeBytes { (pointer: UnsafeRawBufferPointer) -> [UInt8] in
            let buffer = pointer.bindMemory(to: UInt8.self)
            return buffer.map { UInt8($0) }
        }

        let virtualFriend = array.withUnsafeBufferPointer { pointer in
            return VirtualFriend(pointer)
        }

        do {
            let saveData = try Data(contentsOf: saveUrl(for: self.fileName))

            print("Loading save size \(saveData.count)")

            let array = saveData.withUnsafeBytes { (pointer: UnsafeRawBufferPointer) -> [UInt8] in
                let buffer = pointer.bindMemory(to: UInt8.self)
                return buffer.map { UInt8($0) }
            }

            array.withUnsafeBufferPointer { pointer in
                virtualFriend.load_ram(pointer)
            }
        } catch {
            print("Save load error \(error)")
        }

        self.virtualFriend = virtualFriend
        self.audio = try EmulatorAudio()

        self.color = VBColor(foregroundColor: .init(red: 1.0, green: 0.0, blue: 0.0, alpha: 1.0), backgroundColor: .init(red: 0.0, green: 0.0, blue: 0.0, alpha: 1.0))

        fileUrl.stopAccessingSecurityScopedResource()
    }

    func start() {
        guard self.halt else {
            // We are already running
            return
        }

        self.halt = false

        // Run advance frame
        self.runAndWriteFrame()

        self.emulatorQueue.async {
            self.startThread()
        }

        self.audio.start()
    }

    func shutdown() {
        // Kill the emulation loop
        guard !self.halt else {
            // We are already halted
            return
        }

        self.halt = true

        self.audio.stop()

        self.emulatorQueue.async {
            let saveData = Data(self.virtualFriend.save_ram())
            print("Saving size \(saveData.count)")
            do {
                try saveData.write(to: saveUrl(for: self.fileName))
            } catch {
                print("Could not write save \(error)")
            }
        }
    }

    private func startThread() {
        print("Starting emulation")

        let tickRate = Double(AUDIO_FRAMES_PER_LOOP) / Double(SAMPLE_RATE)

        OESetThreadRealtime(tickRate, 0.007, 0.03) // Constants somehow come from bsnes

        var nextFrameTime = OEMonotonicTime()

        while !self.halt {
            autoreleasepool {
                self.runAndWriteFrame()

                nextFrameTime += tickRate

                let currentTime = OEMonotonicTime()

                let timeDifference = currentTime - nextFrameTime

                if timeDifference >= 1.0 {
                    print("Synchronizing time as we are off by \(timeDifference)s");
                    nextFrameTime = currentTime
                }

                OEWaitUntil(nextFrameTime)

                // Tick run loop once to handle events
                CFRunLoopRunInMode(.defaultMode, 0, false)
            }
        }

        print("Stopping emulation")
    }

    private func runAndWriteFrame() {
        let inputs = self.pollInput()

        let frame = self.virtualFriend.run_audio_frame(inputs, AUDIO_FRAMES_PER_LOOP)

        if let frame = frame.video {
            Task {
                let leftImage = frame.left.ciImage(color: self.color)
                let rightImage = frame.right.ciImage(color: self.color)

                // TODO: This should be moved by Metal, not the CPU
               let leftTransformedImage = leftImage.transformed(by: .init(translationX: -self.separation, y: 0))
               let rightTransformedImage = rightImage.transformed(by: .init(translationX: self.separation, y: 0))

               await self.stereoImageChannel.channel.send(StereoImage(left: leftTransformedImage, right: rightTransformedImage))
            }
        }

        self.audio.write(frame: frame)
    }

    private func pollInput() -> FFIGamepadInputs {
        let keyboard = pollKeyboardInput()
        let controller = pollControllerInput()

        return keyboard.merge(controller)
    }

    private func pollKeyboardInput() -> FFIGamepadInputs {
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

    private func pollControllerInput() -> FFIGamepadInputs {
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

enum EmulatorError: Error {
    case audioFormatInit
    case audioBufferInit
}

private func saveUrl(for name: String) -> URL {
    var saveUrl = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
    saveUrl.append(component: "Saves")
    saveUrl.append(component: "\(name).sav")

    return saveUrl
}
