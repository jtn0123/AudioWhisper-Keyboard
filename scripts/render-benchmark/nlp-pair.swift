// Combine actual screenshots without redrawing keyboard content. Run on the development Mac.
import AppKit
import ImageIO
import UniformTypeIdentifiers

let arguments = Array(CommandLine.arguments.dropFirst())
guard arguments.count == 3 else {
    fatalError("usage: swift nlp-pair.swift BEFORE.png AFTER.png OUTPUT.png")
}

func strip(_ path: String) -> CGImage {
    guard let source = CGImageSourceCreateWithURL(URL(fileURLWithPath: path) as CFURL, nil),
          let image = CGImageSourceCreateImageAtIndex(source, 0, nil),
          image.width == 1080, image.height == 2400,
          let crop = image.cropping(to: CGRect(x: 0, y: 1562, width: 1080, height: 140)) else {
        fatalError("Expected a 1080 by 2400 aw_keyboard screenshot")
    }
    return crop
}

let left = strip(arguments[0])
let right = strip(arguments[1])
let width = left.width + right.width
let height = left.height + 56
guard let context = CGContext(data: nil, width: width, height: height, bitsPerComponent: 8,
                              bytesPerRow: width * 4, space: CGColorSpace(name: CGColorSpace.sRGB)!,
                              bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue) else {
    fatalError("Could not allocate screenshot pair")
}
context.setFillColor(NSColor.white.cgColor)
context.fill(CGRect(x: 0, y: 0, width: width, height: height))
context.interpolationQuality = .none
context.draw(left, in: CGRect(x: 0, y: 0, width: left.width, height: left.height))
context.draw(right, in: CGRect(x: left.width, y: 0, width: right.width, height: right.height))
NSGraphicsContext.saveGraphicsState()
NSGraphicsContext.current = NSGraphicsContext(cgContext: context, flipped: false)
let style: [NSAttributedString.Key: Any] = [.font: NSFont.systemFont(ofSize: 28), .foregroundColor: NSColor.black]
("Before" as NSString).draw(at: NSPoint(x: 18, y: height - 44), withAttributes: style)
("After" as NSString).draw(at: NSPoint(x: left.width + 18, y: height - 44), withAttributes: style)
NSGraphicsContext.restoreGraphicsState()
guard let image = context.makeImage(),
      let destination = CGImageDestinationCreateWithURL(URL(fileURLWithPath: arguments[2]) as CFURL,
                                                       UTType.png.identifier as CFString, 1, nil) else {
    fatalError("Could not create screenshot pair")
}
CGImageDestinationAddImage(destination, image, nil)
guard CGImageDestinationFinalize(destination) else { fatalError("Could not save screenshot pair") }
