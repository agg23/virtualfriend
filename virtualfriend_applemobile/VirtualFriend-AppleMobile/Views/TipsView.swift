//
//  TipsView.swift
//  VirtualFriend
//
//  Created by Adam Gastineau on 7/18/24.
//

import SwiftUI
import StoreKit

private let tipsToImages = ["virtualfriend.smalltip": "hand.thumbsup.circle.fill", "virtualfriend.mediumtip": "heart.circle.fill", "virtualfriend.largetip": "creditcard.circle.fill", "virtualfriend.xlargetip": "cat.circle.fill"]

struct TipsView: View {
    @State private var state: TipsLoadingState = .loading
    @State private var showThankYou = false

    var body: some View {
        Group {
            switch self.state {
            case .loading:
                ProgressView()
            case .error:
                Text("Failed to fetch tips")
            case .products(let products):
                StoreView(products: products) { product, icon in
                    let image = tipsToImages[product.id]!
                    Image(systemName: image)
                        .font(.system(size: 40))
                        .symbolRenderingMode(.hierarchical)
                        .foregroundColor(.red)
                }
                #if os(visionOS)
                .productViewStyle(.regular)
                #else
                .productViewStyle(.compact)
                #endif
                .onInAppPurchaseCompletion { _, error in
                    do {
                        let success = try error.get()

                        if case .userCancelled = success {
                            return
                        }

                        self.showThankYou = true
                    } catch {
                        // Do nothing
                    }
                }
            }
        }
        .alert("Thank you!", isPresented: self.$showThankYou) {
            // Alert will automatically be closed by this button
            Button("You're welcome") {}
        } message: {
            Text("Thank you so much for your generous tip. It is greatly appreciated.")
        }
        .task {
            do {
                var products = try await Product.products(for: tipsToImages.keys)

                guard products.count > 0 else {
                    self.state = .error
                    return
                }

                // Place the tips from smallest to largest
                products.sort { a, b in
                    a.price < b.price
                }

                self.state = .products(products)
            } catch {
                self.state = .error

                print("Failed to load tips from App Store")
            }
        }
    }
}

private enum TipsLoadingState {
    case products([Product])
    case loading
    case error
}

#Preview {
    TipsView()
}
