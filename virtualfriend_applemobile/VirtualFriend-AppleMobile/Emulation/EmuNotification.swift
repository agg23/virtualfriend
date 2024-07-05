//
//  EmuNotification.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/5/24.
//

import Foundation

enum EmuNotification {
    case noController
    case controllerConnected
    case none
}

extension EmuNotification {
    var text: String {
        switch self {
        case .noController:
            return "Connect a controller, or press any keyboard key"
        case .controllerConnected:
            return "Controller connected"
        case .none:
            return ""
        }
    }

    var icon: String? {
        switch self {
        case .noController:
            return "gamecontroller.fill"
        case .controllerConnected:
            return "gamecontroller.fill"
        case .none:
            return nil
        }
    }
}
