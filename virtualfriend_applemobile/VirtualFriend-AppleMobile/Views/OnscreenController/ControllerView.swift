//
//  ControllerView.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/6/24.
//

import SwiftUI

struct ControllerView: View {
    @State private var controller: TouchController = TouchController()

    var body: some View {
        HStack {
            DpadView(controller: self.controller, prefix: "left", color: .red, width: 100, height: 100)
                .background(Color.green)

            FaceButtonView(controller: self.controller, name: "start", title: "Start", color: .blue, touchColor: .red)
                .frame(width: 100, height: 100)
        }
        // Use full width as available touch area
        .frame(minWidth: 0, maxWidth: .infinity)
        .overlay {
            TouchGestureView(controller: self.controller)
//                .background(Color.blue)
        }
        .coordinateSpace(.named(self.controller.COORDINATE_SPACE_NAME))
    }
}

#Preview {
    ControllerView()
}
