package com.jtn0123.audiowhisper.gif

import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import com.jtn0123.audiowhisper.rust.RenderProtocol
import java.awt.image.BufferedImage
import java.io.ByteArrayOutputStream
import javax.imageio.ImageIO
import kotlinx.coroutines.runBlocking
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/** Entirely local source: generated GIFs exercise native layout, pixels, selection and idle. */
class FixtureGifSourceTest {
    private class Source {
        val gifs=(0..2).map { Gif("fixture$it","Fixture $it","fixture:$it",(it+1)/2f,"fixture:$it","https://example.invalid/fixture$it") }
        private val bytes=gifs.mapIndexed { i,gif ->
            val image=BufferedImage(4+i,4,BufferedImage.TYPE_INT_RGB)
            for(y in 0 until 4)for(x in 0 until 4+i)image.setRGB(x,y,0x112233*(i+1))
            gif.previewUrl to ByteArrayOutputStream().also {ImageIO.write(image,"gif",it)}.toByteArray()
        }.toMap()
        suspend fun trending()=gifs
        suspend fun search(query: String)=if(query=="empty") emptyList() else gifs.reversed()
        suspend fun preview(url: String)=bytes.getValue(url)
    }
    @Test fun generatedTrendingSearchEmptyAndDecodedPreviews()=runBlocking {
        val source=Source()
        assertEquals(3,source.trending().size)
        assertEquals(source.gifs.reversed(),source.search("fixture"))
        assertTrue(source.search("empty").isEmpty())
        KeyboardRenderer(emptyArray()).use { native ->
            native.theme("""{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet","window":{"background":"#e0e0e0","foreground":"#000000","font-size":"14sp"}}""",emptyArray(),intArrayOf(),floatArrayOf(2f,1f,1f))
            native.update(floatArrayOf(900f,600f,96f,1f,1f),arrayOf("characters","unshifted"),intArrayOf(),floatArrayOf(),emptyArray(),emptyArray())
            val panel=floatArrayOf(5f,2f,96f,0f,50_000_000f,0f,500_000_000f)
            native.panels(panel,intArrayOf(-1,0,0),Array(6){""})
            fun results(status: Int,gifs: List<Gif>) {
                native.gifs(floatArrayOf(status.toFloat(),64f)+gifs.map { it.aspectRatio }.toFloatArray(),
                    arrayOf("Trending","Search GIPHY","Powered by GIPHY",if(status==2)"Retry" else if(status==0)"Loading" else "No results","Send GIF","Trending","Trending","Cats")+gifs.map { it.title },2,LongArray(gifs.size){-(it+1L)})
            }
            results(1,source.trending())
            for((index,gif)in source.trending().withIndex()) {
                assertTrue(native.semanticSnapshot().contains(gif.title))
                val frame=GifFrames(source.preview(gif.previewUrl)).frames().single()
                val decoded=ImageIO.read(frame.bytes.inputStream())
                assertEquals(4,decoded.height)
                val pixels=decoded.getRGB(0,0,decoded.width,decoded.height,null,0,decoded.width)
                assertTrue(native.mediaUpload(-(index+1L),0,decoded.width,decoded.height,100_000_000,true,pixels))
            }
            fun tap(x: Float,y: Float): IntArray {
                native.touch(intArrayOf(0,0,0),longArrayOf(0,1),floatArrayOf(x,y))
                return native.touch(intArrayOf(1,0,0),longArrayOf(0,2),floatArrayOf(x,y))
            }
            assertContentEquals(intArrayOf(RenderProtocol.PANEL_GIF,3,0),tap(20f,200f))
            assertTrue(tap(350f,200f).isEmpty()) // one send at a time
            native.gifSent()
            assertContentEquals(intArrayOf(RenderProtocol.PANEL_GIF,3,1),tap(350f,200f))
            results(2,emptyList());assertTrue(native.semanticSnapshot().contains("Retry"))
            results(1,emptyList());assertTrue(native.semanticSnapshot().contains("No results"))
            results(0,emptyList());assertTrue(native.mediaRequests().isEmpty());assertTrue(native.semanticSnapshot().contains("Loading"))
            results(1,source.trending())
            assertTrue(native.mediaUpload(-3,0,1,1,100_000_000,false,intArrayOf(-1)))
            native.touch(intArrayOf(0,0,0),longArrayOf(0,1),floatArrayOf(850f,500f))
            native.touch(intArrayOf(2,0,0),longArrayOf(0,100_000_000),floatArrayOf(850f,180f))
            native.touch(intArrayOf(3,0,0),longArrayOf(0,200_000_000),floatArrayOf(850f,180f))
            assertTrue(!native.semanticSnapshot().contains("Fixture 2"))
            // An admission already decoding may finish offscreen, but must not animate there.
            assertTrue(native.mediaUpload(-3,1,1,1,100_000_000,true,intArrayOf(-1)))
            assertEquals(-1,native.nextDeadline())
            panel[0]=0f;native.panels(panel,intArrayOf(-1,0,0),Array(6){""})
            assertEquals(-1,native.nextDeadline())
        }
    }
}
