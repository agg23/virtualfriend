//
//  TouchGestureView.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/6/24.
//

import SwiftUI

struct TouchGestureView: UIViewRepresentable {
    let controller: TouchController

    func makeUIView(context: Context) -> some UIView {
        let recognizer = TouchGestureRecognizer(controller: self.controller)

        let view = UIView(frame: .zero)
        view.addGestureRecognizer(recognizer)

        self.controller.view = view

        return view
    }

    func updateUIView(_ uiView: UIViewType, context: Context) {
        // Do nothing
    }
}
