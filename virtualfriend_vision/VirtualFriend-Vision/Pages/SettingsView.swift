//
//  SettingsView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 4/27/24.
//

import SwiftUI

struct SettingsView: View {
    @AppStorage("romDirectoryBookmark") var romDirectoryBookmark: Data?

    @LEDBackgroundColor var ledBackgroundColor;
    @LEDForegroundColor var ledForegroundColor;

    @State var colorImage: UIImage?

    @State var selectFolder = false

    var body: some View {
        NavigationStack {
            VStack {
                Form {
                    Section {
                        Button {
                            self.selectFolder.toggle()
                        } label: {
                            Text("Choose title directory")
                        }
                        .customFileImporter(self.$selectFolder, onOpen: { _, bookmark in
                            self.romDirectoryBookmark = bookmark

                            self.selectFolder = false
                        })
                    }

                    Section("Color") {
                        ColorPicker("Foreground Color", selection: self.$ledForegroundColor, supportsOpacity: false)
                        ColorPicker("Background Color", selection: self.$ledBackgroundColor, supportsOpacity: false)

                        Button {
                            self.ledForegroundColor = Color(red: 1.0, green: 0.0, blue: 0.0)
                            self.ledBackgroundColor = Color(red: 0.0, green: 0.0, blue: 0.0)
                        } label: {
                            Text("Reset Colors")
                        }

                        HStack {
                            Spacer()

                            Group {
                                if let colorImage = self.colorImage {
                                    Image(uiImage: colorImage)
                                        .background {
                                            Color(cgColor: self.ledBackgroundColor.resolve(in: .init()).cgColor)
                                        }
                                } else {
                                    self.ledBackgroundColor
                                }
                            }
                            .aspectRatio(384.0/224.0, contentMode: .fill)

                            Spacer()
                        }
                        .onChange(of: self.ledBackgroundColor) { _, _ in
                            self.regenerateColorImage()

//                            Settings.storeColor(for: Settings.ledBackgroundColorKey, color: self.ledBackgroundColor)
                        }
                        .onChange(of: self.ledForegroundColor) { _, _ in
                            self.regenerateColorImage()

//                            Settings.storeColor(for: Settings.ledForegroundColorKey, color: self.ledForegroundColor)
                        }
                        .onAppear {
                            // Load default colors
//                            if let ledBackgroundColor = Settings.parseStoredColor(for: Settings.ledBackgroundColorKey) {
//                                self.ledBackgroundColor = ledBackgroundColor
//                            }
//
//                            if let ledForegroundColor = Settings.parseStoredColor(for: Settings.ledForegroundColorKey) {
//                                self.ledForegroundColor = ledForegroundColor
//                            }

                            self.regenerateColorImage()
                        }
                    }
                }

            }
            .navigationTitle("Settings")
        }
    }

    func regenerateColorImage() {
        let manifest = FileEntry.getUnknownManifest()
        let ciImage = manifest.left_frame.ciImage(foregroundColor: self.ledForegroundColor.resolve(in: .init()).cgColor, backgroundColor: self.ledBackgroundColor.resolve(in: .init()).cgColor)

        let context = CIContext()
        context.createCGImage(ciImage, from: .init(x: 0, y: 0, width: 384, height: 224))

        // Going directly from CIImage to UIImage doesn't seem to work
        self.colorImage = UIImage(cgImage: context.createCGImage(ciImage, from: .init(x: 0, y: 0, width: 384, height: 224))!)
    }
}

#Preview {
    SettingsView()
}
