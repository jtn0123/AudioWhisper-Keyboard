package com.jtn0123.audiowhisper.gif

import java.awt.image.BufferedImage
import java.io.ByteArrayOutputStream
import javax.imageio.ImageIO
import kotlinx.coroutines.runBlocking
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/** Entirely local source. These GIFs are made in the test, with no downloaded fixture assets. */
class FixtureGifSourceTest {
    private class Source: GifDataSource {
        val gifs=(0..2).map { Gif("fixture$it","Fixture $it","fixture:$it",(it+1)/2f,"fixture:$it","https://example.invalid/fixture$it") }
        private val bytes=gifs.mapIndexed { i,gif ->
            val image=BufferedImage(4+i,4,BufferedImage.TYPE_INT_RGB)
            for(y in 0 until 4)for(x in 0 until 4+i)image.setRGB(x,y,0x112233*(i+1))
            gif.previewUrl to ByteArrayOutputStream().also {ImageIO.write(image,"gif",it)}.toByteArray()
        }.toMap()
        override suspend fun trending()=gifs
        override suspend fun search(query: String)=if(query=="empty") emptyList() else gifs.reversed()
        override suspend fun preview(url: String)=bytes.getValue(url)
    }
    @Test fun generatedTrendingSearchEmptyAndDecodedPreviews()= runBlocking {
        val source=Source()
        assertEquals(3,source.trending().size)
        assertEquals(source.gifs.reversed(),source.search("fixture"))
        assertTrue(source.search("empty").isEmpty())
        for(gif in source.trending()) {
            val frames=GifFrames(source.preview(gif.previewUrl)).frames().toList()
            assertEquals(1,frames.size)
            assertEquals(4,ImageIO.read(frames.single().bytes.inputStream()).height)
        }
    }
}
