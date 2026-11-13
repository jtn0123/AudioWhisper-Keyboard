package com.jtn0123.audiowhisper.gif

import java.io.ByteArrayOutputStream
import java.io.IOException

/** GIF framing only. Each standalone frame is decoded by Android's BitmapFactory, on IO. */
internal class GifFrames(private val bytes: ByteArray) {
    data class Frame(val bytes: ByteArray,val x: Int,val y: Int,val width: Int,val height: Int,val delayMs: Int,val disposal: Int)
    private var offset=0
    private fun byte(): Int {if(offset>=bytes.size) throw IOException("Truncated GIF");return bytes[offset++].toInt() and 255}
    private fun word(): Int = byte() or (byte() shl 8)
    private fun skip(n: Int) {if(n<0 || offset+n>bytes.size) throw IOException("Truncated GIF");offset+=n}
    private fun blocks() {while(true){val size=byte();if(size==0)return;skip(size)}}
    private val header: ByteArray
    val width: Int
    val height: Int
    init {
        if(bytes.size>4*1024*1024 || bytes.size<13 || String(bytes,0,3,Charsets.US_ASCII)!="GIF") throw IOException("Invalid GIF")
        offset=6;width=word();height=word()
        if(width !in 1..1024 || height !in 1..1024) throw IOException("GIF dimensions exceed decode bound")
        val packed=byte();skip(2)
        if(packed and 128 != 0) skip(3*(1 shl ((packed and 7)+1)))
        header=bytes.copyOfRange(0,offset)
    }
    fun frames(): Sequence<Frame> = sequence {
        var control=byteArrayOf(0x21,0xf9.toByte(),4,0,10,0,0,0)
        var count=0
        while(offset<bytes.size && count<64) {
            when(byte()) {
                0x3b -> return@sequence
                0x21 -> {
                    if(byte()==0xf9) {
                        val start=offset-2
                        if(byte()!=4) throw IOException("Invalid GIF control")
                        skip(5);control=bytes.copyOfRange(start,offset)
                        if(control[7]!=0.toByte()) throw IOException("Invalid GIF control")
                    } else blocks()
                }
                0x2c -> {
                    val start=offset-1
                    val x=word();val y=word();val w=word();val h=word();val packed=byte()
                    if(w==0 || h==0 || x+w>width || y+h>height) throw IOException("Invalid GIF frame")
                    if(packed and 128 != 0) skip(3*(1 shl ((packed and 7)+1)))
                    byte();blocks()
                    val descriptor=bytes.copyOfRange(start,offset)
                    descriptor[1]=0;descriptor[2]=0;descriptor[3]=0;descriptor[4]=0
                    val localHeader=header.copyOf()
                    localHeader[6]=w.toByte();localHeader[7]=(w shr 8).toByte()
                    localHeader[8]=h.toByte();localHeader[9]=(h shr 8).toByte()
                    val delay=(control[4].toInt() and 255) or ((control[5].toInt() and 255) shl 8)
                    val disposal=(control[3].toInt() ushr 2) and 7
                    val output=ByteArrayOutputStream(localHeader.size+control.size+descriptor.size+1)
                    output.write(localHeader);output.write(control);output.write(descriptor);output.write(0x3b)
                    yield(Frame(output.toByteArray(),x,y,w,h,if(delay<2)100 else delay*10,disposal))
                    count++;control=byteArrayOf(0x21,0xf9.toByte(),4,0,10,0,0,0)
                }
                else -> throw IOException("Invalid GIF block")
            }
        }
    }
}
