//
//  EmuHeaderOverlayView.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/4/24.
//

import SwiftUI

struct EmuHeaderOverlayView: View {
    @LEDBackgroundColor private var ledBackgroundColor

    let title: String

    let isImmersed: Bool

    let resetTimer: () -> Void
    let onBack: () -> Void
    let onImmersive: () -> Void
    let onRestart: () -> Void

    let onCreateSavestate: () -> Void
    let onOpenSavestates: () -> Void

    var body: some View {
        // Line up with buttons in top bar
        let buttonOutsidePadding = 16.0

        VStack {
            self.header

            Spacer()

            #if !os(visionOS)
            HStack {
                self.createSavestateButton
                    .padding(.leading, buttonOutsidePadding)

                Spacer()

                self.loadSavestateButton
                    .padding(.trailing, buttonOutsidePadding)
            }
            .symbolRenderingMode(.hierarchical)
            .labelStyle(.iconOnly)
            .tint(.white)
            .symbolVariant(.circle.fill)
            .font(.title2)

            .frame(maxWidth: .infinity)
            .padding(.top, 16.0)
            .padding(.bottom, 8.0)
            // We insert .secondary background so we can ensure the control buttons are visible over the user configuable background color
            .background(self.ledBackgroundColor.isDark ? .secondary : Color.black.opacity(0.4))
            #endif
        }
    }

    @ViewBuilder
    private var header: some View {
        // Attempt to match NavigationBar padding
        #if os(visionOS)
        let buttonPadding = 20.0
        let bottomPadding = 16.0
        #else
        let buttonPadding = 8.0
        let bottomPadding = 8.0
        #endif

        HStack {
            Button {
                self.resetTimer()

                self.onBack()
            } label: {
                Label {
                    Text("Back to Library")
                } icon: {
                    Image(systemName: Icon.back)
                }
            }
            .help("Back to Library")
            .padding(.leading, buttonPadding)

            #if os(visionOS)
            Button {
                self.resetTimer()

                self.onImmersive()
            } label: {
                Label {
                    Text(self.isImmersed ? "Leave immersive space" : "Enter immersive space")
                } icon: {
                    Image(systemName: Icon.immersive)
                }
            }
            .help(self.isImmersed ? "Leave immersive space" : "Enter immersive space")
            .padding(.leading, buttonPadding)

            // Placeholder button
            Button {

            } label: {
                Image(systemName: Icon.back)
            }
            .hidden()
            .padding(.leading, buttonPadding)
            #endif

            Text(self.title)
                #if os(visionOS)
                .font(.title)
                #else
                .font(.headline)
                #endif
                .lineLimit(1)
                .truncationMode(.tail)
                .foregroundColor(.white)
                // Let text expand to fill full width of view
                .frame(maxWidth: .infinity)

            #if os(visionOS)
            self.createSavestateButton
                .padding(.trailing, buttonPadding)

            self.loadSavestateButton
                .padding(.trailing, buttonPadding)
            #endif

            Button {
                self.resetTimer()

                self.onRestart()
            } label: {
                Label {
                    Text("Restart")
                } icon: {
                    Image(systemName: Icon.restart)
                }
            }
            .help("Restart")
            .padding(.trailing, buttonPadding)
        }
        .symbolRenderingMode(.hierarchical)
        .labelStyle(.iconOnly)
        .buttonBorderShape(.circle)
        .controlSize(.large)
        #if !os(visionOS)
        .tint(.white)
        .symbolVariant(.circle.fill)
        .font(.largeTitle)
        #endif
        .padding(.bottom, bottomPadding)
        .padding(.top, buttonPadding)
        #if os(visionOS)
        // Vision looks bad with the (white) .secondary, so we use the same color as our details view
        .background(Color.black.opacity(0.4))
        #else
        // We insert .secondary background so we can ensure the control buttons are visible over the user configuable background color
        .background(self.ledBackgroundColor.isDark ? .secondary : Color.black.opacity(0.4))
        #endif
    }

    @ViewBuilder
    private var createSavestateButton: some View {
        Button {
            self.onCreateSavestate()
        } label: {
            Label {
                Text("Create Savestate")
            } icon: {
                Image(systemName: Icon.savestateCreate)
            }
        }
        .help("Create Savestate")
    }

    @ViewBuilder
    private var loadSavestateButton: some View {
        Button {
            self.onOpenSavestates()
        } label: {
            Label {
                Text("Load Savestate")
            } icon: {
                Image(systemName: Icon.savestateLoad)
            }
        }
        .help("Load Savestates")
    }
}
