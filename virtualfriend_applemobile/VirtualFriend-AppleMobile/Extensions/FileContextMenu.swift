//
//  FileContextMenu.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 8/8/26.
//

import SwiftUI

struct FileContextMenu: ViewModifier {
    @Environment(FileImporter.self) private var fileImporter

    let url: URL

    func body(content: Content) -> some View {
        content.contextMenu {
            Button(role: .destructive) {
                self.fileImporter.delete(at: self.url)
            } label: {
                Label("Delete", systemImage: "trash")
            }
        }
    }
}

extension View {
    func fileContextMenu(_ url: URL) -> some View {
        modifier(FileContextMenu(url: url))
    }
}
