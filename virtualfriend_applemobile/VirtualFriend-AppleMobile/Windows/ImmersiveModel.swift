//
//  ImmersiveModel.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 8/12/24.
//

import SwiftUI

#if os(visionOS)
@Observable class ImmersiveModel {
    private var openAction: OpenImmersiveSpaceAction?
    private var dismissAction: DismissImmersiveSpaceAction?

    var isImmersed = false

    func initialize(openAction: OpenImmersiveSpaceAction, dismissAction: DismissImmersiveSpaceAction) {
        self.openAction = openAction
        self.dismissAction = dismissAction
    }

    func open() async {
        guard let openAction = self.openAction else {
            fatalError("ImmersiveModel not initialized")
        }

        switch await openAction(id: "ImmersiveSpace") {
        case .opened:
            self.isImmersed = true
        case .error:
            fallthrough
        case .userCancelled:
            fallthrough
        @unknown default:
            self.isImmersed = false
        }
    }

    func dismiss() async {
        guard let dismissAction = self.dismissAction else {
            fatalError("ImmersiveModel not initialized")
        }

        self.isImmersed = false
        await dismissAction()
    }
}
#else
@Observable class ImmersiveModel {
    // Stub for non-visionOS
    var isImmersed = false

    func open() async {}
    func dismiss() async {}
}
#endif
