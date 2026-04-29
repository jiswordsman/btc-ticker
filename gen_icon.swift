import Cocoa

func createIcon(size: CGFloat, color: NSColor, filename: String) {
    let canvasSize = NSSize(width: size, height: size)
    let fontSize = size * 0.78
    let image = NSImage(size: canvasSize)
    image.lockFocus()
    
    NSColor.clear.set()
    NSRect(origin: .zero, size: canvasSize).fill()

    let font = NSFont.systemFont(ofSize: fontSize, weight: .bold)
    let attributes: [NSAttributedString.Key: Any] = [
        .font: font,
        .foregroundColor: color
    ]
    
    let btc = NSString(string: "₿")
    let btcSize = btc.size(withAttributes: attributes)
    let rect = NSRect(
        x: (size - btcSize.width) / 2,
        y: (size - btcSize.height) / 2 - size * 0.03,
        width: btcSize.width,
        height: btcSize.height
    )
    
    btc.draw(in: rect, withAttributes: attributes)
    
    image.unlockFocus()
    
    if let tiff = image.tiffRepresentation, let bitmap = NSBitmapImageRep(data: tiff) {
        if let data = bitmap.representation(using: .png, properties: [:]) {
            try? data.write(to: URL(fileURLWithPath: filename))
        }
    }
}

let bitcoinOrange = NSColor(red: 0.965, green: 0.612, blue: 0.055, alpha: 1.0)
let upGreen = NSColor(red: 0.0, green: 0.75, blue: 0.53, alpha: 1.0)
let downRed = NSColor(red: 1.0, green: 0.3, blue: 0.31, alpha: 1.0)

createIcon(size: 1024, color: bitcoinOrange, filename: "src-tauri/icons/icon.png")
createIcon(size: 32, color: bitcoinOrange, filename: "src-tauri/icons/32x32.png")
createIcon(size: 128, color: bitcoinOrange, filename: "src-tauri/icons/128x128.png")
createIcon(size: 256, color: bitcoinOrange, filename: "src-tauri/icons/128x128@2x.png")

createIcon(size: 64, color: bitcoinOrange, filename: "src-tauri/icons/tray-neutral.png")
createIcon(size: 64, color: upGreen, filename: "src-tauri/icons/tray-up.png")
createIcon(size: 64, color: downRed, filename: "src-tauri/icons/tray-down.png")
