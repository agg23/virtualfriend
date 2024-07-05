//
//  Toast.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/4/24.
//

import SwiftUI

enum ToastText {
    case content(text: String, icon: String?)
    case none
}

extension ToastText: Equatable {

}

struct ToastWrapper<T: View>: View {
    @Binding var text: ToastText

    @ViewBuilder let content: () -> T

    // We copy text presence to this state so we can set it with withAnimation
    @State private var show: Bool = false
    @State private var cachedText: ToastText?

    init(text: Binding<ToastText>, @ViewBuilder content: @escaping () -> T) {
        self._text = text
        self.content = content
    }

    var body: some View {
        let (displayText, icon) = self.extractText()

        ZStack {
            self.content()

            VStack {
                ToastBody(text: displayText, icon: icon)
                    .padding(.top, self.show ? 20 : -150)
                    .animation(.easeInOut(duration: 1.0), value: self.show)

                Spacer()
            }
            .onTapGesture {
                self.text = .none
            }
        }
        .onChange(of: self.text) { oldValue, newValue in
            withAnimation {
                let newShow = newValue != .none

                if newShow && oldValue != .none {
                    // We need to animate this item out, and the next in
                    self.show = false

                    DispatchQueue.main.asyncAfter(deadline: .now() + 1.0) {
                        // Update text now
                        self.cachedText = newValue

                        self.show = true
                    }
                } else {
                    self.show = newShow

                    guard newShow else {
                        return
                    }

                    // If we're showing, save our text and icon to prevent rendering issues
                    self.cachedText = newValue
                }
            }
        }
    }

    func extractText() -> (String, String?) {
        let text = self.cachedText ?? self.text

        if case .content(text: let text, icon: let icon) = text {
            return (text, icon)
        }

        return ("", nil)
    }
}

private struct ToastBody: View {
    @Environment(\.colorScheme) private var colorScheme: ColorScheme

    let text: String
    let icon: String?

    var body: some View {
        let backgroundColor = self.colorScheme == .dark ? Color(white: 0.15) : Color(white: 0.85)

        Group {
            HStack {
                if let icon = self.icon {
                    Image(systemName: icon)
                }

                Text(self.text)
                    .multilineTextAlignment(.center)
            }
        }
        .padding()
        .background {
            RoundedRectangle(cornerRadius: 1000)
                .fill(backgroundColor)
        }
    }
}

#Preview {
    struct PreviewWrapper: View {
        @State var text: ToastText = .none

        var body: some View {
            ToastWrapper(text: self.$text) {
                Button {
                    if self.text == .none {
                        self.text = .content(text: "Connect a controller, or press any keyboard key", icon: "gamecontroller.fill")
                    } else {
                        self.text = .none
                    }
                } label: {
                    Text("Foo")
                }
            }
        }
    }

    return PreviewWrapper()
}

#Preview {
    ZStack {
        Color.clear

        ToastBody(text: "This is the toast content", icon: "gamecontroller.fill")
    }
}
