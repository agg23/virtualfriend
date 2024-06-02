//
//  Hash.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 1/28/24.
//

import Foundation
import CommonCrypto

func hashOfFile(atUrl url: URL) -> String? {
    let bufferSize = 4096

    let file: FileHandle

    do {
        file = try FileHandle(forReadingFrom: url)
    } catch {
        return nil
    }

    defer {
        file.closeFile()
    }

    var context = CC_MD5_CTX()
    CC_MD5_Init(&context)

    while autoreleasepool(invoking: {
        let data = file.readData(ofLength: bufferSize)
        if data.count > 0 {
            data.withUnsafeBytes { buffer in
                _ = CC_MD5_Update(&context, buffer.baseAddress, numericCast(data.count))
            }
            return true
        } else {
            return false
        }
    }) { }

    var digest = [UInt8](repeating: 0, count: Int(CC_MD5_DIGEST_LENGTH))
    CC_MD5_Final(&digest, &context)

    return digest.map { String(format: "%02hhx", $0) }.joined()
}
