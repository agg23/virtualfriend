//
//  Files.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/28/24.
//

import Foundation

extension FileManager {
    func getFilesWithExtension(at url: URL, fileExtension: String) -> [URL] {
        let files: [URL]

        do {
            files = try FileManager.default.contentsOfDirectory(at: url, includingPropertiesForKeys: [.isRegularFileKey])
        } catch {
            return []
        }

        return files.filter { url in
            url.pathExtension == fileExtension
        }
    }
}

func saveUrl(for name: String) -> URL {
    var saveUrl = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
    saveUrl.append(component: "Saves")
    saveUrl.append(component: "\(name).sav")

    return saveUrl
}

func savestateBaseUrl(for name: String) -> URL {
    var saveUrl = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
    saveUrl.append(component: "Savestates")
    saveUrl.append(component: "\(name)")

    return saveUrl
}

func savestateFileName(for name: String, date: Date) -> String {
    let dateFormatter = savestateDateFormatter()

    let formattedData = dateFormatter.string(from: date).replacingOccurrences(of: "/", with: ":")

    return "\(formattedData) \(name).ss"
}
