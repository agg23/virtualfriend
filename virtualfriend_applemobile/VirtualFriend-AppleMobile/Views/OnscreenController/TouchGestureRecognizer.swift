//
//  TouchGestureRecognizer.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/6/24.
//

import UIKit

class TouchGestureRecognizer: UIGestureRecognizer {
    let controller: TouchController

    var activeTouches: Set<UITouch> = .init()

    init(controller: TouchController) {
        print("Init")
        self.controller = controller

        super.init(target: nil, action: nil)
    }

    override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent) {
//        print("Touches began \(touches)")
        self.activeTouches.formUnion(touches)
        self.controller.update(touches: self.activeTouches)
    }

    override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent) {
//        print("Touches moved \(touches)")
        // Actual objects in the set haven't changed, but the positions of the existing events may have moved
        self.controller.update(touches: self.activeTouches)
    }

    override func touchesEnded(_ touches: Set<UITouch>, with event: UIEvent) {
//        print("Touches ended \(touches)")
        self.activeTouches.subtract(touches)
        self.controller.update(touches: self.activeTouches)
    }

    override func touchesCancelled(_ touches: Set<UITouch>, with event: UIEvent) {
        self.activeTouches.subtract(touches)
        self.controller.update(touches: self.activeTouches)
    }
}
