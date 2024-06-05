//
//  MainRouter.swift
//  VirtualFriend-AppleMobile
//
//  Created by Adam Gastineau on 6/4/24.
//

import Foundation

@Observable class MainRouter {
    var currentRoute: Route = .main

    enum Route {
        case main
        case emulator(url: URL)
    }
}
