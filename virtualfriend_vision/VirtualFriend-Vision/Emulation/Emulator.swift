//
//  Emulator.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 4/14/24.
//

import SwiftUI
import AsyncAlgorithms
import GameController
import AVFAudio

class Emulator {
    private let actor: EmulatorActor
    private let audioEngine: AVAudioEngine
//    private let audioNode: AVAudioPlayerNode
    private var audioNode: AVAudioSourceNode!
    private let audioConverter: AVAudioConverter

    private let audioInputBuffer: AVAudioPCMBuffer
    private let audioOutputBuffer0: AVAudioPCMBuffer
    private let audioOutputBuffer1: AVAudioPCMBuffer

    private var inputBufferLength: Int = 0

    var stereoImageChannel = AsyncImageChannel()

    var executingTask: Task<(), Error>?

    var prevFrameTime: TimeInterval?

    var separation: Binding<Double>?

    var enableSound: Bool = false

    init(fileUrl: URL) throws {
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

        self.actor = EmulatorActor(virtualFriend: virtualFriend)

        fileUrl.stopAccessingSecurityScopedResource()

        self.audioEngine = AVAudioEngine()

        guard let outputFormat = AVAudioFormat(standardFormatWithSampleRate: AVAudioSession.sharedInstance().sampleRate, channels: 2),
              let inputFormat = AVAudioFormat(commonFormat: .pcmFormatInt16, sampleRate: 41667, channels: 2, interleaved: false),
              let audioConverter = AVAudioConverter(from: inputFormat, to: outputFormat) else {
            throw EmulatorError.audioFormatInit
        }

        self.audioConverter = audioConverter

        let outputBufferCapacity = AVAudioFrameCount(4096)

        guard let audioInputBuffer = AVAudioPCMBuffer(pcmFormat: inputFormat, frameCapacity: AVAudioFrameCount(20000)),
              let audioOutputBuffer0 = AVAudioPCMBuffer(pcmFormat: outputFormat, frameCapacity: outputBufferCapacity),
              let audioOutputBuffer1 = AVAudioPCMBuffer(pcmFormat: outputFormat, frameCapacity: outputBufferCapacity) else {
            throw EmulatorError.audioBufferInit
        }

        self.audioInputBuffer = audioInputBuffer
        self.audioOutputBuffer0 = audioOutputBuffer0
        self.audioOutputBuffer1 = audioOutputBuffer1

        // Capture the number of frames we expect given the resampling
//        audioConverter.convert(to: self.audioOutputBuffer0, error: nil) { packetCount, status in
//            self.inputBufferLength = Int(packetCount)
//
//            status.pointee = .endOfStream
//
//            return nil
//        }

        print("Expecting \(self.inputBufferLength) frames at a time")

        audioConverter.reset()

        self.audioNode = AVAudioSourceNode(format: inputFormat, renderBlock: { _isSilence, _timestamp, frameCount, outputBuffer -> OSStatus in
            print(frameCount)

            self.executingTask = Task {
                let frame = await self.runAudioGroupedFrame(UInt(frameCount), interval: 0)

                if Task.isCancelled {
                    return
                }

                self.renderAudioBuffer(frame, buffer: self.audioOutputBuffer0)
            }

            let listBuffer = UnsafeMutableAudioBufferListPointer(outputBuffer)

//            guard let inputChannelData = self.audioOutputBuffer0.floatChannelData else {
//                fatalError("Could not get int16 channel data")
//            }

            guard let inputChannelData = self.audioInputBuffer.int16ChannelData else {
                fatalError("Could not get int16 channel data")
            }

            let leftInputChannel = inputChannelData[0]
            let rightInputChannel = inputChannelData[1]

            let leftOutputChannel = UnsafeMutableBufferPointer<Int16>(listBuffer[0])
            let rightOutputChannel = UnsafeMutableBufferPointer<Int16>(listBuffer[1])

            for i in 0..<Int(frameCount) {
                leftOutputChannel[i] = leftInputChannel[i]
                rightOutputChannel[i] = rightInputChannel[i]
            }

            return 0
        })
        self.audioEngine.attach(self.audioNode)


        self.audioEngine.connect(self.audioNode, to: self.audioEngine.mainMixerNode, format: outputFormat)
    }

    func start() {
        do {
            try AVAudioSession.sharedInstance().setCategory(.playback, mode: .default, policy: .longFormAudio, options: [])

            try AVAudioSession.sharedInstance().setPreferredIOBufferDuration(0.005)
            try AVAudioSession.sharedInstance().setAllowHapticsAndSystemSoundsDuringRecording(true)

            try AVAudioSession.sharedInstance().setActive(true)
        } catch {
            print(error)
        }

        self.audioNode.reset()

        // Initial silence buffer
        self.audioInputBuffer.frameLength = AVAudioFrameCount(10000)

        self.executingTask = Task {
            let frame = await self.runAudioGroupedFrame(UInt(487), interval: 0)
            self.renderAudioBuffer(frame, buffer: self.audioOutputBuffer0)

            do {
                try self.audioEngine.start()

    //            self.audioNode.play()
            } catch {
                print(error)
            }
        }

//        Task {
//            // Run two sets of audio frames to speed up scheduling of buffers
//            let frame0 = await self.runAudioGroupedFrame(UInt(self.inputBufferLength), interval: 0)
//            let frame1 = await self.runAudioGroupedFrame(UInt(self.inputBufferLength), interval: 0)
//            self.renderAudioBuffer(frame0, buffer: self.audioOutputBuffer0)
//            self.renderAudioBuffer(frame1, buffer: self.audioOutputBuffer1)
//        }
    }

    func stop() {
        self.executingTask?.cancel()
        self.executingTask = nil

        self.audioEngine.stop()
    }

    func enableSound(_ enable: Bool) {
        self.enableSound = enable

        if !enable {
            // Turn off sound, clear buffer
            guard let inputBuffer = self.audioInputBuffer.int16ChannelData else {
                return
            }

            let channel0 = inputBuffer[0]
            let channel1 = inputBuffer[1]

            for i in 0..<20000 {
                channel0[Int(i)] = 0
                channel1[Int(i)] = 0
            }
        }
    }

    private func runAudioGroupedFrame(_ bufferSize: UInt, interval: TimeInterval) async -> FFIFrame {
        let frameTime = Date().timeIntervalSince1970

        let shouldBe = 1.0/41667.0 * Double(bufferSize)
        let actual = frameTime - (self.prevFrameTime ?? frameTime)

        print(actual, bufferSize, "Should be \(shouldBe)")

        if shouldBe <= actual {
            print("Overran audio buffer")
        }

        self.prevFrameTime = frameTime

        let inputs = self.pollInput()

        let frame = await self.actor.runAudioFrame(with: inputs, bufferSize: bufferSize)

        if let frame = frame.video {
            let leftImage = rustVecToCIImage(frame.left)
            let rightImage = rustVecToCIImage(frame.right)

            // TODO: This should be flipped by Metal, not the CPU
            let leftTransformedImage = leftImage.transformed(by: .init(scaleX: 1, y: -1).translatedBy(x: -(self.separation?.wrappedValue ?? 0.0), y: 0))
            let rightTransformedImage = rightImage.transformed(by: .init(scaleX: 1, y: -1).translatedBy(x: (self.separation?.wrappedValue ?? 0.0), y: 0))

            await self.stereoImageChannel.channel.send(StereoImage(left: leftTransformedImage, right: rightTransformedImage))
        }

        print("Diff \(Date().timeIntervalSince1970 - interval)")

        return frame
    }

    private func renderAudioBuffer(_ frame: FFIFrame, buffer: AVAudioPCMBuffer) {
        guard self.enableSound else {
            return
        }

        guard let inputBuffer = self.audioInputBuffer.int16ChannelData else {
            return
        }

        var conversionError: NSError?

//        self.audioConverter.convert(to: buffer, error: &conversionError) { packetCount, status in
//            let channel0 = inputBuffer[0]
//            let channel1 = inputBuffer[1]
//
//            for (i, (left, right)) in zip(frame.audio_left, frame.audio_right).enumerated() {
//                channel0[i] = left
//                channel1[i] = right
//            }
//
//            status.pointee = .hasData
//
//            self.audioInputBuffer.frameLength = AVAudioFrameCount(frame.audio_left.len())
//            return self.audioInputBuffer
//        }

        let channel0 = inputBuffer[0]
        let channel1 = inputBuffer[1]

        for (i, (left, right)) in zip(frame.audio_left, frame.audio_right).enumerated() {
            channel0[i] = left
            channel1[i] = right
        }

        self.audioInputBuffer.frameLength = AVAudioFrameCount(frame.audio_left.len())

        if let error = conversionError {
            print(error, error.userInfo)
        }

//        self.runAndScheduleFrame(buffer: buffer)
    }

//    private func runAndScheduleFrame(buffer: AVAudioPCMBuffer) {
//        let start = Date().timeIntervalSince1970
//
//        Task {
//            let frame = await self.runAudioGroupedFrame(UInt(self.inputBufferLength), interval: start)
//
////            await self.audioNode.scheduleBuffer(buffer, completionCallbackType: .dataConsumed)
//
//            self.renderAudioBuffer(frame, buffer: buffer)
//        }
//    }

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

/// Used to prevent concurrent access to Rust code
private actor EmulatorActor {
    let virtualFriend: VirtualFriend

    init(virtualFriend: VirtualFriend) {
        self.virtualFriend = virtualFriend
    }

    func runAudioFrame(with inputs: FFIGamepadInputs, bufferSize: UInt) -> FFIFrame {
        return self.virtualFriend.run_audio_frame(inputs, bufferSize)
    }
}

enum EmulatorError: Error {
    case audioFormatInit
    case audioBufferInit
}
