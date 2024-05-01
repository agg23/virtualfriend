//
//  CustomFileImporter.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 4/27/24.
//

import SwiftUI

struct CustomFileImporter: ViewModifier {
    var isPresented: Binding<Bool>
    var onOpen: (_ url: URL, _ bookmark: Data?) -> Void

    func body(content: Content) -> some View {
        content
            .fileImporter(isPresented: isPresented, allowedContentTypes: [.folder]) { result in
                switch result {
                case .success(let url):
                    print("Selected \(url)")

                    guard url.startAccessingSecurityScopedResource() else {
                        print("Could not obtain security scope")
                        return
                    }

                    defer { url.stopAccessingSecurityScopedResource() }

                    // Update bookmark
                    let bookmarkData = try? url.bookmarkData()
                    self.onOpen(url, bookmarkData)
                case .failure(let failure):
                    print("Failed to open folder: \(failure)")
                }
            }
    }
}

extension View {
    func customFileImporter(_ isPresented: Binding<Bool>, onOpen: @escaping (_ url: URL, _ bookmark: Data?) -> Void) -> some View {
        modifier(CustomFileImporter(isPresented: isPresented, onOpen: onOpen))
    }
}
