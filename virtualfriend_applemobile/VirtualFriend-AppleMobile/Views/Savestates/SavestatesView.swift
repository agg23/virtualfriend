//
//  SavestatesView.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 9/1/24.
//

import SwiftUI

struct SavestatesView: View {
    @Environment(\.dismiss) private var dismiss

    @State private var savestates: [Date: [DatedSavestateUrl]] = [:]
    @State private var sortedDates: [Date] = []

    let fileName: String
    let dateFormatter: DateFormatter

    init(fileName: String) {
        self.fileName = fileName

        let dateFormatter = DateFormatter()
        dateFormatter.dateStyle = .full
        dateFormatter.timeStyle = .none

        self.dateFormatter = dateFormatter
    }

    var body: some View {
        NavigationView {
            List {
                ForEach(self.sortedDates, id: \.self) { date in
                    let daySavestates = self.savestates[date]!

                    Section(dateFormatter.string(from: date)) {
                        ForEach(daySavestates, id: \.url) { savestate in
                            SavestateRowView(datedUrl: savestate)
                        }
                    }
                }
            }
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel", role: .cancel) {
                        dismiss()
                    }
                }
            }
        }
        .onAppear {
            self.scanSavestates()
        }
    }

    func scanSavestates() {
        let dateFormatter = savestateDateFormatter()

        let fileName = String(self.fileName.split(separator: ".")[0])

        do {
            let savestateUrls = try FileManager.default.contentsOfDirectory(at: savestateBaseUrl(for: fileName), includingPropertiesForKeys: [.isRegularFileKey])


            let savestates: [DatedSavestateUrl] = savestateUrls.filter({ $0.pathExtension == "ss" }).compactMap { savestateUrl in
                let dateString = savestateUrl.lastPathComponent.replacingOccurrences(of: " \(fileName).ss", with: "")

                guard let date = dateFormatter.date(from: dateString) else {
                    return nil
                }

                return DatedSavestateUrl(date: date, url: savestateUrl)
            }

            self.savestates = savestates.groupByDay()

            self.sortedDates = self.savestates.keys.sorted(by: >)
        } catch {
            print("Could not load savestate directory contents \(error)")
        }
    }
}

struct DatedSavestateUrl {
    let date: Date
    let url: URL
}

extension Array where Element == DatedSavestateUrl {
    func groupByDay() -> [Date: [DatedSavestateUrl]] {
        var dictionary = [Date: [DatedSavestateUrl]]()

        for savestate in self {
            let components = Calendar.current.dateComponents([.year, .month, .day], from: savestate.date)
            let date = Calendar.current.date(from: components)!

            var urls = dictionary[date] ?? []
            urls.append(savestate)
            dictionary[date] = urls
        }

        return dictionary
    }
}

#Preview {
    SavestatesView(fileName: "foo")
}
