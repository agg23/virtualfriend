//
//  MainRouter.swift
//  VirtualFriend-AppleMobile
//
//  Created by Adam Gastineau on 6/4/24.
//

import Foundation

@Observable class MainRouter {
    var currentRoute: Route = .main

    /// The currently active, selected file, if any
    var selectedFile: URL?

    enum Route {
        case main
        case emulator(url: URL)
    }
}
