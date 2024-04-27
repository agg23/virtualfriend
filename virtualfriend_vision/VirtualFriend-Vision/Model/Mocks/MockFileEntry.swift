//
//  MockFileEntry.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 4/27/24.
//

import Foundation

func MOCK_FILE_ENTRIES() -> [FileEntry] {
    return [
        FileEntry(url: URL(string: "foo/Foo.vb")!, hash: "foo"),
        FileEntry(url: URL(string: "bar/Bar.vb")!, hash: "bar"),
        FileEntry(url: URL(string: "foobar/Foobar.vb")!, hash: "foobar"),
        FileEntry(url: URL(string: "barfoo/Barfoo.vb")!, hash: "barfoo"),
    ]
}

func MOCK_FILE_ENTRIES_WITH_MANIFESTS() -> [FileEntryWithManifest] {
    return MOCK_FILE_ENTRIES().map { entry in
        FileEntryWithManifest(entry: entry, manifest: FFIManifest(left_frame: .init(), right_frame: .init(), metadata: Optional(FFIMetadata(title: .init(entry.url.lastPathComponent), developer: .init("agg23"), publisher: .init("Virtual Boy"), year: .init("2024"), region: .init()))))
    }
}

func MOCK_FILE_ENTRY() -> FileEntry {
    return MOCK_FILE_ENTRIES()[0]
}

func MOCK_FILE_ENTRY_WITH_MANIFEST() -> FileEntryWithManifest {
    return MOCK_FILE_ENTRIES_WITH_MANIFESTS()[0]
}
