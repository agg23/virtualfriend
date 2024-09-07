//
//  Date.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 9/7/24.
//

import Foundation

func savestateDateFormatter() -> DateFormatter {
    let dateFormatter = DateFormatter()
    dateFormatter.dateFormat = "yyyy-MM-dd HH-mm-ss"

    return dateFormatter
}
