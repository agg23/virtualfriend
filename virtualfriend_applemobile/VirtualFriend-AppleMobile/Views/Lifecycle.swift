//
//  Lifecycle.swift
//  VirtualFriend
//
//  Created by Eric Lewis on 3/15/24.
//

import SwiftUI

struct LifecycleViewModifier: ViewModifier {
    @Environment(\.scenePhase) private var scenePhase

    var mount: () -> Void
    var unmount: () -> Void

    func body(content: Content) -> some View {
        content
            .task {
                // Grouped together similar to onAppear/onDisappear
                self.mount()
                await Task.streamAwaitCancellation()
                self.unmount()
            }
            .onChange(of: self.scenePhase) { prevPhase, newPhase in
                // This is finicky. Particularly check if we're transitioning from inactive to background, because
                // only then are we _sure_ we just killed the window
                if prevPhase == .inactive && newPhase == .background {
                    self.unmount()
                }
        }
    }
}

extension View {
    func mount(_ f: @escaping () -> Void, unmount: @escaping () -> Void) -> some View {
        self.modifier(LifecycleViewModifier(mount: f, unmount: unmount))
    }
}
