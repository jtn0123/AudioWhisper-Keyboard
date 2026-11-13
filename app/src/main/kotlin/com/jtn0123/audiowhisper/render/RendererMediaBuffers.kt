package com.jtn0123.audiowhisper.render

import android.content.Context
import android.content.ContentUris
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.media.ThumbnailUtils
import android.util.Size
import com.jtn0123.audiowhisper.rust.KeyboardRenderer
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardFileStorage
import dev.patrickgold.florisboard.ime.clipboard.provider.ItemType
import kotlinx.coroutines.*

/** Decodes once on admission, never while painting. Native requests visible, uncached ids only. */
internal class RendererMediaBuffers(private val context: Context, private val native: KeyboardRenderer, private val changed: () -> Unit) {
    private val scope = CoroutineScope(SupervisorJob()+Dispatchers.Main)
    private val pending = mutableMapOf<Long,Job>()
    private val failed = mutableSetOf<Long>()
    private var cohort = emptySet<Long>()
    fun update(snapshot: RendererClipboard?, gif: RendererGif? = null) {
        val current = snapshot?.entries.orEmpty().map { it.id }.toSet()
        if (current != cohort) { failed.retainAll(current); cohort=current }
        val wanted = native.mediaRequests().toSet()
        for ((id,job) in pending.toMap()) if (id !in wanted) { job.cancel();pending.remove(id) }
        for (id in wanted) {
            if(id in pending || id in failed) continue
            val item = snapshot?.entries?.find { it.id==id }
            val result = gif?.ids?.indexOf(id)?.takeIf { it>=0 }?.let { gif.gifs[it] }
            if(item==null && result==null) continue
            pending[id]=scope.launch {
                try {
                    if(result!=null) {
                        withContext(Dispatchers.IO) {com.jtn0123.audiowhisper.gif.GifSearch.source?.preview(result.previewUrl)?.let { RendererGifDecoder.upload(native,id,it) }}
                        changed()
                        return@launch
                    }
                    requireNotNull(item)
                    val bitmap=withContext(Dispatchers.IO) {
                        val file=ClipboardFileStorage.getFileForId(context,ContentUris.parseId(requireNotNull(item.uri)))
                        if(item.type==ItemType.VIDEO) ThumbnailUtils.createVideoThumbnail(file,Size(256,256),null)
                        else {
                            val bounds=BitmapFactory.Options().apply { inJustDecodeBounds=true }
                            BitmapFactory.decodeFile(file.path,bounds)
                            val options=BitmapFactory.Options().apply { inSampleSize=(maxOf(bounds.outWidth,bounds.outHeight)/256).coerceAtLeast(1) }
                            requireNotNull(BitmapFactory.decodeFile(file.path,options))
                        }
                    }
                    try {
                        val pixels=IntArray(bitmap.width*bitmap.height)
                        bitmap.getPixels(pixels,0,bitmap.width,0,0,bitmap.width,bitmap.height)
                        if(native.mediaUpload(id,0,bitmap.width,bitmap.height,0,true,pixels)) changed()
                    } finally {bitmap.recycle()}
                }catch(e: java.io.IOException){failed.add(id)}catch(e: IllegalArgumentException){failed.add(id)}
                finally {pending.remove(id)}
            }
        }
        if(snapshot==null && gif==null) {failed.clear();cohort=emptySet();com.jtn0123.audiowhisper.gif.GifImages.forgetPreviews()}
    }
    fun close(){scope.cancel();pending.clear();failed.clear()}
}
