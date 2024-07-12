//
//  SettingsView.swift
//  VirtualFriend-Vision
//
//  Created by Adam Gastineau on 4/27/24.
//

import SwiftUI

struct SettingsView: View {
    @LEDBackgroundColor var ledBackgroundColor;
    @LEDForegroundColor var ledForegroundColor;

    @EnableSound var enableSound
    @Enable3D var enable3D

    @State var colorImage: UIImage?

    @State var selectFolder = false

    var body: some View {
        NavigationStack {
            VStack {
                Form {
                    Section("Gameplay") {
                        Toggle("Enable Audio", isOn: self.$enableSound)

                        #if os(visionOS)
                        Toggle(isOn: self.$enable3D) {
                            Text("Enable 3D")
                            Text("Enables the stereoscopic 3D effect")
                        }
                        #endif
                    }

                    Section("Controls") {
                        NavigationLink("Keyboard") {
                            KeyboardSettingsView()
                        }
                    }

                    Section("Color") {
                        ColorPicker("Foreground Color", selection: self.$ledForegroundColor, supportsOpacity: false)
                        ColorPicker("Background Color", selection: self.$ledBackgroundColor, supportsOpacity: false)

                        HStack {
                            Spacer()

                            Group {
                                if let colorImage = self.colorImage {
                                    Image(uiImage: colorImage)
                                        // Allow there to be padding around the image
                                        .resizable()
                                        .background {
                                            Color(cgColor: self.ledBackgroundColor.resolve(in: .init()).cgColor)
                                        }
                                } else {
                                    self.ledBackgroundColor
                                }
                            }
                            .aspectRatio(384.0/224.0, contentMode: .fit)
                            .frame(maxHeight: 400)

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

                    Section("Color Presets") {
                        self.presetButton("Default", foregroundColor: Color(red: 1.0, green: 0.0, blue: 0.0), backgroundColor: Color(red: 0.0, green: 0.0, blue: 0.0))

                        self.presetButton("Greyscale", foregroundColor: Color(red: 1.0, green: 1.0, blue: 1.0), backgroundColor: Color(red: 0.0, green: 0.0, blue: 0.0))

                        self.presetButton("Inverted", foregroundColor: Color(red: 0.0, green: 0.0, blue: 0.0), backgroundColor: Color(red: 1.0, green: 0.0, blue: 0.0))

                        self.presetButton("Greyscale Inverted", foregroundColor: Color(red: 0.0, green: 0.0, blue: 0.0), backgroundColor: Color(red: 1.0, green: 1.0, blue: 1.0))

                        self.presetButton("Game Boy", foregroundColor: Color(red: 8/255.0, green: 24/255.0, blue: 32/255.0), backgroundColor: Color(red: 224/255.0, green: 248/255.0, blue: 208/255.0))
                    }

                    Section("About") {
                        Text("VirtualFriend is open source software provided under the MIT license. Contributions are greatly appreciated.")

                        Link("View on GitHub", destination: URL(string: "https://github.com/agg23/virtualfriend")!)
                    }

                    Section {
                        Text("If you like my work, please consider leaving me a tip.")
                    }

                    Section("Licenses") {
                        Text("VirtualFriend for Apple platforms relies on the following OSS libraries:")

                        Link("swift-async-algorithms", destination: URL(string: "https://github.com/apple/swift-async-algorithms")!)

                        Link("swift-collections", destination: URL(string: "https://github.com/apple/swift-collections")!)

                        Link("TPCircularBuffer", destination: URL(string: "https://github.com/michaeltyson/TPCircularBuffer")!)

                        Text("Additional libraries used as reference, which may have code present in the repository:")

                        Link("rustual-boy", destination: URL(string: "https://github.com/emu-rs/rustual-boy")!)

                        Link("OpenEmu-SDK", destination: URL(string: "https://github.com/OpenEmu/OpenEmu-SDK")!)
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
        let color = VBColor(foregroundColor: foregroundColor.resolve(in: .init()).cgColor, backgroundColor: backgroundColor.resolve(in: .init()).cgColor)
        return manifest.left_frame.uiImage(color: color)
    }
}

#Preview {
    SettingsView()
}
