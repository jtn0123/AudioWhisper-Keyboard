package com.jtn0123.audiowhisper.gif

import java.io.ByteArrayOutputStream
import javax.imageio.ImageIO
import java.awt.image.BufferedImage
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import java.io.IOException

/** Fixtures generated locally in the test, never fetched or stored in the repository. */
class GifFramesTest {
    private fun fixture(): ByteArray {
        val image=BufferedImage(3,2,BufferedImage.TYPE_INT_RGB)
        image.setRGB(1,1,0xff00ff)
        val output=ByteArrayOutputStream()
        val writer=ImageIO.getImageWritersByFormatName("gif").next()
        ImageIO.createImageOutputStream(output).use { stream ->
            writer.output=stream
            val parameters=writer.defaultWriteParam
            parameters.progressiveMode=javax.imageio.ImageWriteParam.MODE_DISABLED
            writer.write(null,javax.imageio.IIOImage(image,null,null),parameters)
        }
        writer.dispose()
        return output.toByteArray()
    }
    @Test fun decodesGeneratedGifFramesWithoutNetwork() {
        val bytes=fixture()
        assertEquals(0xff00ff,ImageIO.read(bytes.inputStream()).getRGB(1,1) and 0xffffff)
        val parsed=GifFrames(bytes)
        assertEquals(3,parsed.width);assertEquals(2,parsed.height)
        val frame=parsed.frames().single()
        val decoded=ImageIO.read(frame.bytes.inputStream())
        assertEquals(3,decoded.width);assertEquals(2,decoded.height)
        assertEquals(0xff00ff,decoded.getRGB(1,1) and 0xffffff)
    }
    @Test fun animationControlsAndDisposalStayWithEachFrame() {
        val bytes=fixture()
        val static=GifFrames(bytes).frames().single().bytes
        val header=static.copyOfRange(0,13+3*(1 shl ((static[10].toInt() and 7)+1)))
        val imageStart=header.size+8
        val descriptor=static.copyOfRange(imageStart,static.size-1)
        val output=ByteArrayOutputStream()
        output.write(header)
        for(disposal in 1..3) {
            output.write(byteArrayOf(0x21,0xf9.toByte(),4,(disposal shl 2).toByte(),(disposal*10).toByte(),0,0,0))
            output.write(descriptor)
        }
        output.write(0x3b)
        val frames=GifFrames(output.toByteArray()).frames().toList()
        assertEquals(listOf(100,200,300),frames.map { it.delayMs })
        assertEquals(listOf(1,2,3),frames.map { it.disposal })
        frames.forEach {assertEquals(3,ImageIO.read(it.bytes.inputStream()).width)}
    }
    @Test fun refusesTruncationAndOversizedDimensions() {
        val bytes=fixture()
        assertFailsWith<IOException>{GifFrames(bytes.copyOf(11))}
        val large=bytes.copyOf();large[6]=0;large[7]=8
        assertFailsWith<IOException>{GifFrames(large)}
        assertFailsWith<IOException>{GifFrames(bytes.copyOf(bytes.size-4)).frames().toList()}
    }
}
