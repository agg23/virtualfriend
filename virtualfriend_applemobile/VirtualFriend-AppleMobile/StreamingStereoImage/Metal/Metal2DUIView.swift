//
//  Metal2DUIView.swift
//  VirtualFriend-AppleMobile
//
//  Created by Adam Gastineau on 6/3/24.
//

//
//  Based on NESScreenView.swift
//  nes-emu-ios
//
//  Created by Tom Salvo on 6/9/20.
//  Copyright © 2020 Tom Salvo.
//
//  Permission is hereby granted, free of charge, to any person obtaining a copy
//  of this software and associated documentation files (the "Software"), to deal
//  in the Software without restriction, including without limitation the rights
//  to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//  copies of the Software, and to permit persons to whom the Software is
//  furnished to do so, subject to the following conditions:
//
//  The above copyright notice and this permission notice shall be included in all
//  copies or substantial portions of the Software.
//
//  THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//  IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//  FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//  AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//  LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//  OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//  SOFTWARE.

import UIKit
import MetalKit
import os

#if os(iOS)
final class Metal2DUIView: MTKView, MTKViewDelegate
{
    private let queue: DispatchQueue = DispatchQueue.init(label: "renderQueue", qos: .userInteractive)
    private var hasSuspended: Bool = false
    private let rgbColorSpace: CGColorSpace = CGColorSpaceCreateDeviceRGB()
    private let context: CIContext
    private let commandQueue: MTLCommandQueue
    private var nearestNeighborRendering: Bool
    private var integerScaling: Bool
    private var currentScale: CGFloat = 1.0
    private var viewportOffset: CGPoint = CGPoint.zero
    private var screenTransform: CGAffineTransform = CGAffineTransform.identity
    static private let elementLength: Int = 4
    static private let bitsPerComponent: Int = 8

    init()
    {
        let dev: MTLDevice = MTLCreateSystemDefaultDevice()!
        let commandQueue = dev.makeCommandQueue()!
        self.context = CIContext.init(mtlCommandQueue: commandQueue, options: [.cacheIntermediates: false])
        self.commandQueue = commandQueue
//        self.nearestNeighborRendering = UserDefaults.standard.bool(forKey: Settings.nearestNeighborRenderingKey)
//        self.integerScaling = UserDefaults.standard.bool(forKey: Settings.integerScalingKey)
        self.nearestNeighborRendering = false
        self.integerScaling = true

        super.init(frame: .zero, device: nil)

        self.device = dev
        self.autoResizeDrawable = true
        self.isPaused = true
        self.enableSetNeedsDisplay = false
        self.framebufferOnly = false
        self.delegate = self
        self.isOpaque = true
        self.clearsContextBeforeDrawing = false

        NotificationCenter.default.addObserver(self, selector: #selector(appResignedActive), name: UIApplication.willResignActiveNotification, object: nil)
        NotificationCenter.default.addObserver(self, selector: #selector(appBecameActive), name: UIApplication.didBecomeActiveNotification, object: nil)
    }

    required init(coder: NSCoder) {
        fatalError("Unimplemented")
    }

    deinit
    {
        NotificationCenter.default.removeObserver(self)
    }

    var image: CIImage = CIImage.empty()
    {
        didSet
        {

            self.queue.async { [weak self] in
                self?.draw()
            }
        }
    }

    // MARK: - MTKViewDelegate

    func mtkView(_ view: MTKView, drawableSizeWillChange size: CGSize)
    {
//        let exactScale: CGFloat = size.width / self.drawableSize.width
//        self.currentScale = self.integerScaling ? floor(exactScale) : exactScale
//        self.viewportOffset = self.integerScaling ? CGPoint(x: (size.width - (self.drawableSize.width * self.currentScale)) * 0.5, y: (size.height - (self.drawableSize.height * self.currentScale)) * 0.5) : CGPoint.zero
//
//        let t1: CGAffineTransform = CGAffineTransform(scaleX: self.currentScale, y: self.currentScale)
//        let t2: CGAffineTransform = self.integerScaling ? CGAffineTransform(translationX: self.viewportOffset.x, y: self.viewportOffset.y) : CGAffineTransform.identity
//        self.screenTransform = t1.concatenating(t2)
    }

    func draw(in view: MTKView)
    {
        guard let safeCurrentDrawable = self.currentDrawable,
            let safeCommandBuffer = self.commandQueue.makeCommandBuffer()
        else
        {
            return
        }

        let transformedImage = self.image.transformed(by: .init(translationX: -self.image.extent.origin.x, y: -self.image.extent.origin.y))

//        if self.nearestNeighborRendering
//        {
//            transformedImage = transformedImage.samplingNearest().transformed(by: self.screenTransform)
//        }
//        else
//        {
//            transformedImage = transformedImage.transformed(by: self.screenTransform)
//        }

        let renderDestination = CIRenderDestination(width: Int(self.drawableSize.width), height: Int(self.drawableSize.height), pixelFormat: self.colorPixelFormat, commandBuffer: safeCommandBuffer) {
            () -> MTLTexture in return safeCurrentDrawable.texture
        }

        do
        {
            let _ = try self.context.startTask(toRender: transformedImage, to: renderDestination)
        }
        catch
        {
            os_log("%@", error.localizedDescription)
        }

        safeCommandBuffer.present(safeCurrentDrawable)
        safeCommandBuffer.commit()
    }

    @objc private func appResignedActive()
    {
        self.queue.suspend()
        self.hasSuspended = true
    }

    @objc private func appBecameActive()
    {
        if self.hasSuspended
        {
            self.queue.resume()
            self.hasSuspended = false
        }
    }
}
#endif
