package com.jtn0123.audiowhisper.render

import android.graphics.*
import com.jtn0123.audiowhisper.gif.GifFrames
import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive

/** Decode/composite once per cache admission; Rust owns playback clocks and admission limits. */
internal object RendererGifDecoder {
    suspend fun upload(native: KeyboardRenderer,id: Long,bytes: ByteArray) {
        val gif=GifFrames(bytes)
        val bitmap=Bitmap.createBitmap(gif.width,gif.height,Bitmap.Config.ARGB_8888)
        val canvas=Canvas(bitmap)
        val scale=minOf(1f,128f/maxOf(gif.width,gif.height))
        val width=(gif.width*scale).toInt().coerceAtLeast(1)
        val height=(gif.height*scale).toInt().coerceAtLeast(1)
        val iterator=gif.frames().iterator()
        var index=0
        try {
            while(iterator.hasNext()) {
                currentCoroutineContext().ensureActive()
                val frame=iterator.next()
                val previous=if(frame.disposal==3) bitmap.copy(Bitmap.Config.ARGB_8888,false) else null
                val decoded=BitmapFactory.decodeByteArray(frame.bytes,0,frame.bytes.size) ?: throw java.io.IOException("GIF frame decode failed")
                try {canvas.drawBitmap(decoded,frame.x.toFloat(),frame.y.toFloat(),null)}finally{decoded.recycle()}
                val sampled=Bitmap.createScaledBitmap(bitmap,width,height,true)
                val pixels=IntArray(width*height)
                try{sampled.getPixels(pixels,0,width,0,0,width,height)}finally{if(sampled!==bitmap)sampled.recycle()}
                if(!native.mediaUpload(id,index++,width,height,frame.delayMs*1_000_000L,!iterator.hasNext(),pixels)) {previous?.recycle();break}
                when(frame.disposal) {
                    2 -> canvas.drawRect(frame.x.toFloat(),frame.y.toFloat(),(frame.x+frame.width).toFloat(),(frame.y+frame.height).toFloat(),Paint().apply{xfermode=PorterDuffXfermode(PorterDuff.Mode.CLEAR)})
                    3 -> previous?.let {canvas.drawColor(Color.TRANSPARENT,PorterDuff.Mode.CLEAR);canvas.drawBitmap(it,0f,0f,null)}
                }
                previous?.recycle()
            }
        }finally{bitmap.recycle()}
    }
}
