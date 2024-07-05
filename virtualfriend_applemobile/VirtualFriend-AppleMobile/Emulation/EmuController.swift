//
//  EmuController.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/5/24.
//

import GameController

@Observable class EmuController {
    var notification: EmuNotification = .none {
        didSet {
            print("Setting \(self.notification)")
            self.timer?.invalidate()

            // Keep .noController onscreen
            guard self.notification != .none && self.notification != .noController else {
                return
            }

            self.timer = Timer.scheduledTimer(withTimeInterval: 2, repeats: false) { _ in
                self.notification = .none
            }
        }
    }

    private var timer: Timer?

    private var connectObserver: (any NSObjectProtocol)?
    private var disconnectObserver: (any NSObjectProtocol)?

    init() {
        self.connectObserver = NotificationCenter.default.addObserver(forName: .GCControllerDidConnect, object: nil, queue: nil) { _ in
            if GCController.controllers().count == 1 {
                // First connected
                self.notification = .controllerConnected
            }
        }

        self.disconnectObserver = NotificationCenter.default.addObserver(forName: .GCControllerDidDisconnect, object: nil, queue: nil) { _ in
            if GCController.controllers().count < 1 {
                self.notification = .noController
            }
        }

        // We don't have a controller at startup, mark no controller
        if GCController.controllers().count < 1 {
            self.notification = .noController
        }

        GCKeyboard.coalesced?.keyboardInput?.keyChangedHandler = { input, button, keycode, pressed in
            if self.notification == .noController && pressed {
                self.notification = .none
            }
        }
    }

    deinit {
        if let connectObserver = self.connectObserver {
            NotificationCenter.default.removeObserver(connectObserver)
        }

        if let disconnectObserver = self.disconnectObserver {
            NotificationCenter.default.removeObserver(disconnectObserver)
        }
    }
}
