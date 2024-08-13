//
//  Task.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 3/15/24.
//

import Foundation

extension Task where Failure == Never, Success == Never {
    /// A stream that just sits and waits for cancellation
    static func streamAwaitCancellation() async {
        let stream = AsyncStream<Int> { _ in }
        for await _ in stream {}
    }
}
