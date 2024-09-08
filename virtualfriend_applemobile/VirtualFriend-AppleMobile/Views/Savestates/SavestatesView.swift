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
    let emulator: Emulator
    let dateFormatter: DateFormatter

    init(fileName: String, emulator: Emulator) {
        self.fileName = fileName
        self.emulator = emulator

        let dateFormatter = DateFormatter()
        dateFormatter.dateStyle = .full
        dateFormatter.timeStyle = .none

        self.dateFormatter = dateFormatter
    }

    var body: some View {
        NavigationView {
            Group {
                if self.savestates.isEmpty {
                    Text("No savestates. Please create a savestate from the game overlay")
                        .font(.system(size: 24))
                        .multilineTextAlignment(.center)
                        .foregroundStyle(.secondary)
                        #if os(visionOS)
                        .frame(width: 500)
                        #endif
                } else {
                    List {
                        ForEach(self.sortedDates, id: \.self) { date in
                            let daySavestates = self.savestates[date]!

                            Section(dateFormatter.string(from: date)) {
                                ForEach(daySavestates, id: \.url) { savestate in
                                    SavestateRowView(datedUrl: savestate) { unparsedSavestate in
                                        self.dismiss()

                                        self.emulator.apply(savestate: unparsedSavestate)
                                    }
                                }
                            }
                        }
                    }
                }
            }
            .navigationTitle("Savestates")
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel", role: .cancel) {
                        self.dismiss()
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

extension DatedSavestateUrl: Comparable {
    static func < (lhs: DatedSavestateUrl, rhs: DatedSavestateUrl) -> Bool {
        lhs.date < rhs.date
    }
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

        for day in dictionary.keys {
            dictionary[day]?.sort(by: >)
        }

        return dictionary
    }
}
