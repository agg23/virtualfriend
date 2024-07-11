//
//  CGPoint.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/10/24.
//

import Foundation

extension CGPoint {
    static func +(lhs: CGPoint, rhs: CGPoint) -> CGPoint {
        CGPoint(x: lhs.x + rhs.x, y: lhs.y + rhs.y)
    }
}
