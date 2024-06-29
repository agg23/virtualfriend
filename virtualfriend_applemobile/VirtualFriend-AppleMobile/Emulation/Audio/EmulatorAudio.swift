//
//  EmulatorAudio.swift
//  VirtualFriend-AppleMobile
//
//  Created by Adam Gastineau on 6/29/24.
//

import AVFAudio

/// 41.667kHz
private let VB_SAMPLE_RATE = 41667
private let CHANNEL_COUNT = 2
/// Int16
private let BYTES_PER_VALUE = 2

class EmulatorAudio {
    private var ringBuffer: OERingBuffer
    private let audioEngine: AVAudioEngine
    private var audioNode: AVAudioSourceNode!

    init() throws {
        // Store at most 1/10th of a second
        let frameSampleCount = Int((Double(VB_SAMPLE_RATE) * 0.1).rounded())
        let length = CHANNEL_COUNT * BYTES_PER_VALUE * frameSampleCount

        self.ringBuffer = OERingBuffer(length: UInt(length))
        self.ringBuffer.discardPolicy = .oldest
        self.ringBuffer.anticipatesUnderflow = true

        // Audio engine
        self.audioEngine = AVAudioEngine()

        guard let inputFormat = AVAudioFormat(commonFormat: .pcmFormatInt16, sampleRate: 41667, channels: 2, interleaved: true),
              let outputFormat = AVAudioFormat(standardFormatWithSampleRate: AVAudioSession.sharedInstance().sampleRate, channels: 2) else {
            throw EmulatorError.audioFormatInit
        }

        // Audio render callback runs at the VB's 41.667kHz sample rate. This will be automatically converted to the current system's output sample rate and format
        self.audioNode = AVAudioSourceNode(format: inputFormat, renderBlock: { _isSilence, _timestamp, frameCount, outputBuffer -> OSStatus in
            let listBuffer = UnsafeMutableAudioBufferListPointer(outputBuffer)

            let requestedBytes = UInt(Int(frameCount) * BYTES_PER_VALUE * CHANNEL_COUNT)

            let readBytes = self.ringBuffer.read(listBuffer[0].mData, maxLength: requestedBytes)

//            print("Read \(readBytes) bytes, available \(self.ringBuffer.availableBytes / 2)")

            let readDiff = requestedBytes - readBytes

            if readDiff > 0 {
                // Fill remaining parts of buffer with zeros
                memset(listBuffer[0].mData! + Int(readBytes), 0, Int(readDiff))
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

            try self.audioEngine.start()
        } catch {
            print("Failed to set up AVAudioSession: \(error)")
        }
    }

    func stop() {
        self.audioEngine.stop()
    }

    func write(frame: FFIFrame) {
        // Copy audio data to buffer for ringbuffer
        var dataBuffer = Data(count: frame.audio_left.len() * BYTES_PER_VALUE * CHANNEL_COUNT)

        dataBuffer.withUnsafeMutableBytes { pointer in
            pointer.withMemoryRebound(to: Int16.self) { buffer in
                for (i, (left, right)) in zip(frame.audio_left, frame.audio_right).enumerated() {
                    buffer[i * 2] = left
                    buffer[i * 2 + 1] = right
                }

            }
        }

        let _ = dataBuffer.withUnsafeBytes { pointer in
            // 2 channels * 2 bytes per value
            self.ringBuffer.write(pointer.baseAddress!, maxLength: UInt(frame.audio_left.len() * 2 * 2))
//            print("Wrote \(frame.audio_left.len()) frames, available \(self.ringBuffer.availableBytes / 2)")
        }
    }
}
