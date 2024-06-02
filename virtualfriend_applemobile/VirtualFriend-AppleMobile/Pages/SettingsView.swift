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
                        }
                        .onChange(of: self.ledForegroundColor) { _, _ in
                            self.regenerateColorImage()
                        }
                        .onAppear {
                            self.regenerateColorImage()
                        }
                    }

                    Section("Presets") {
                        self.presetButton("Default", foregroundColor: Color(red: 1.0, green: 0.0, blue: 0.0), backgroundColor: Color(red: 0.0, green: 0.0, blue: 0.0))

                        self.presetButton("Greyscale", foregroundColor: Color(red: 1.0, green: 1.0, blue: 1.0), backgroundColor: Color(red: 0.0, green: 0.0, blue: 0.0))

                        self.presetButton("Inverted", foregroundColor: Color(red: 0.0, green: 0.0, blue: 0.0), backgroundColor: Color(red: 1.0, green: 0.0, blue: 0.0))
                    }
                }

            }
            .navigationTitle("Settings")
        }
    }

    @ViewBuilder
    func presetButton(_ title: String, foregroundColor: Color, backgroundColor: Color) -> some View {
        Button {
            self.ledForegroundColor = foregroundColor
            self.ledBackgroundColor = backgroundColor
        } label: {
            HStack {
                Text(title)

                Spacer()

                Image(uiImage: self.generateImage(foregroundColor: foregroundColor, backgroundColor: backgroundColor))
                    .resizable()
                    .aspectRatio(contentMode: .fit)
            }
        }
        .frame(height: 64)
    }

    func regenerateColorImage() {
        self.colorImage = self.generateImage(foregroundColor: self.ledForegroundColor, backgroundColor: self.ledBackgroundColor)
    }

    func generateImage(foregroundColor: Color, backgroundColor: Color) -> UIImage {
        let manifest = FileEntry.getUnknownManifest()
        let ciImage = manifest.left_frame.ciImage(foregroundColor: foregroundColor.resolve(in: .init()).cgColor, backgroundColor: backgroundColor.resolve(in: .init()).cgColor)

        let context = CIContext()
        context.createCGImage(ciImage, from: .init(x: 0, y: 0, width: 384, height: 224))

        // Going directly from CIImage to UIImage doesn't seem to work
        return UIImage(cgImage: context.createCGImage(ciImage, from: .init(x: 0, y: 0, width: 384, height: 224))!)
    }
}

#Preview {
    SettingsView()
}
