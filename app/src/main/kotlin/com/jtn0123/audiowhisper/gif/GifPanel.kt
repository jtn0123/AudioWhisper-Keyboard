package com.jtn0123.audiowhisper.gif

import android.graphics.drawable.Animatable
import android.graphics.drawable.Drawable
import android.widget.ImageView
import android.widget.Toast
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.sizeIn
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.staggeredgrid.LazyVerticalStaggeredGrid
import androidx.compose.foundation.lazy.staggeredgrid.StaggeredGridCells
import androidx.compose.foundation.lazy.staggeredgrid.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.State
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.viewinterop.AndroidView
import com.jtn0123.audiowhisper.dictation.DictationFields
import dev.patrickgold.florisboard.editorInstance
import dev.patrickgold.florisboard.ime.ImeUiMode
import dev.patrickgold.florisboard.ime.keyboard.FlorisImeSizing
import dev.patrickgold.florisboard.ime.theme.FlorisImeUi
import dev.patrickgold.florisboard.keyboardManager
import kotlinx.coroutines.launch
import org.florisboard.lib.compose.florisHorizontalScroll
import org.florisboard.lib.snygg.ui.SnyggChip
import org.florisboard.lib.snygg.ui.SnyggIcon
import org.florisboard.lib.snygg.ui.SnyggIconButton
import org.florisboard.lib.snygg.ui.SnyggRow

private val TileShape = RoundedCornerShape(6.dp)
private val Placeholder = Color.Gray.copy(alpha = 0.2f)
private const val COLUMNS = 3
private const val MIN_ASPECT = 0.5f
private const val MAX_ASPECT = 2.5f

/** The GIF tab of the media panel: a search field, quick searches and a grid of GIFs. */
@Composable
fun GifPanel(modifier: Modifier = Modifier) {
    val editorInstance by LocalContext.current.editorInstance()
    val results by GifSearch.results.collectAsState()
    val shown by GifSearch.shown.collectAsState()
    val keyboardManager by LocalContext.current.keyboardManager()
    LaunchedEffect(Unit) {
        val private = keyboardManager.activeState.isIncognitoMode ||
            DictationFields.isSecret(editorInstance.activeInfo.inputAttributes.variation)
        GifSearch.offerTypedWord(editorInstance.activeContent.textBeforeSelection, private)
    }
    Column(modifier = modifier.fillMaxWidth()) {
        GifHeader(shown)
        GifCategories(shown)
        Box(modifier = Modifier.fillMaxWidth().weight(1f), contentAlignment = Alignment.Center) {
            when (val result = results) {
                GifSearch.Results.Loading -> CircularProgressIndicator(Modifier.size(28.dp), strokeWidth = 2.dp)
                is GifSearch.Results.Failed -> Message("${result.message} Tap to try again.") { GifSearch.load(shown) }
                is GifSearch.Results.Loaded -> when {
                    result.gifs.isEmpty() -> Message("No GIFs for “$shown”.")
                    else -> GifGrid(result.gifs)
                }
            }
        }
    }
}

/** Tapping the field opens the keyboard to type a search; ✕ goes back to trending GIFs. */
@Composable
private fun GifHeader(shown: String) {
    val keyboardManager by LocalContext.current.keyboardManager()
    val searched = shown != GifSearch.TRENDING
    Row(
        modifier = Modifier.fillMaxWidth().height(FlorisImeSizing.smartbarHeight).padding(horizontal = 8.dp, vertical = 6.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Row(
            modifier = Modifier
                .weight(1f)
                .fillMaxHeight()
                .clip(RoundedCornerShape(50))
                .background(Placeholder)
                .clickable {
                    GifSearch.startTyping()
                    keyboardManager.activeState.imeUiMode = ImeUiMode.TEXT
                }
                .padding(horizontal = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(Icons.Default.Search, contentDescription = null, modifier = Modifier.size(18.dp))
            Text(
                text = if (searched) shown else "Search GIPHY",
                modifier = Modifier.weight(1f).padding(start = 8.dp).alpha(if (searched) 1f else 0.6f),
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            if (searched) {
                Icon(
                    Icons.Default.Close,
                    contentDescription = "Back to trending GIFs",
                    modifier = Modifier.size(18.dp).clickable { GifSearch.load(GifSearch.TRENDING) },
                )
            }
        }
        // GIPHY's terms ask for this to be shown wherever its GIFs are.
        Text("Powered by GIPHY", modifier = Modifier.padding(start = 8.dp).alpha(0.6f), fontSize = 11.sp)
    }
}

@Composable
private fun GifCategories(shown: String) {
    val chips by GifSearch.chips.collectAsState()
    SnyggRow(
        elementName = FlorisImeUi.ClipboardFilterRow.elementName,
        modifier = Modifier.fillMaxWidth(),
        clickAndSemanticsModifier = Modifier.florisHorizontalScroll(),
    ) {
        for (category in chips) {
            SnyggChip(
                elementName = FlorisImeUi.ClipboardFilterChip.elementName,
                attributes = mapOf("state" to if (category == shown) "active" else "inactive"),
                onClick = { GifSearch.load(category) },
                text = category,
            )
        }
    }
}

/** Tapping a GIF sends it and goes back to the keyboard. */
@Composable
private fun GifGrid(gifs: List<Gif>) {
    val context = LocalContext.current
    val keyboardManager by context.keyboardManager()
    val editorInstance by context.editorInstance()
    val scope = rememberCoroutineScope()
    var sending by remember { mutableStateOf<String?>(null) }
    LazyVerticalStaggeredGrid(
        columns = StaggeredGridCells.Fixed(COLUMNS),
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(4.dp),
        verticalItemSpacing = 4.dp,
        horizontalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        items(gifs, key = { it.id }) { gif ->
            GifTile(gif, sending = sending == gif.id) {
                if (sending != null) return@GifTile
                sending = gif.id
                scope.launch {
                    val outcome = GifSender.send(context, editorInstance, gif)
                    sending = null
                    if (outcome != GifSender.Outcome.FAILED) {
                        GifSearch.sent(gif, incognito = keyboardManager.activeState.isIncognitoMode)
                    }
                    outcome.message()?.let { Toast.makeText(context, it, Toast.LENGTH_SHORT).show() }
                    if (outcome != GifSender.Outcome.FAILED) keyboardManager.activeState.imeUiMode = ImeUiMode.TEXT
                }
            }
        }
    }
}

private fun GifSender.Outcome.message(): String? = when (this) {
    GifSender.Outcome.SENT -> null
    GifSender.Outcome.LINKED -> "This app can't take GIFs, so its link was typed instead"
    GifSender.Outcome.FAILED -> "Couldn't send that GIF"
}

@Composable
private fun GifTile(gif: Gif, sending: Boolean, onClick: () -> Unit) {
    val image by produceState<Drawable?>(null, gif.previewUrl) { value = GifImages.preview(gif.previewUrl) }
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .aspectRatio(gif.aspectRatio.coerceIn(MIN_ASPECT, MAX_ASPECT))
            .clip(TileShape)
            .background(Placeholder),
    ) {
        image?.let { drawable ->
            AndroidView(
                factory = { ImageView(it).apply { scaleType = ImageView.ScaleType.CENTER_CROP } },
                modifier = Modifier.fillMaxSize(),
                update = { view ->
                    view.setImageDrawable(drawable)
                    (drawable as? Animatable)?.start()
                },
            )
        }
        // On top of the image view, so the tap reaches Compose.
        Box(Modifier.matchParentSize().clickable(onClickLabel = gif.title.ifBlank { "Send GIF" }, onClick = onClick))
        if (sending) {
            Box(Modifier.matchParentSize().background(Color.Black.copy(alpha = 0.5f)), contentAlignment = Alignment.Center) {
                CircularProgressIndicator(Modifier.size(24.dp), color = Color.White, strokeWidth = 2.dp)
            }
        }
    }
}

@Composable
private fun Message(text: String, onClick: (() -> Unit)? = null) {
    Text(
        text = text,
        modifier = Modifier
            .fillMaxWidth()
            .let { if (onClick != null) it.clickable(onClick = onClick) else it }
            .padding(24.dp),
        textAlign = TextAlign.Center,
    )
}

/**
 * Replaces the suggestion strip while a GIF search is typed: the keys type into this field and
 * enter (or the search button) runs the search. Back returns to the GIFs unchanged.
 */
@Composable
fun GifSearchBar() {
    val keyboardManager by LocalContext.current.keyboardManager()
    val query by GifSearch.query.collectAsState()
    val buttonSize = Modifier.sizeIn(maxHeight = FlorisImeSizing.smartbarHeight).aspectRatio(1f)
    SnyggRow(
        elementName = FlorisImeUi.Smartbar.elementName,
        modifier = Modifier.fillMaxWidth().height(FlorisImeSizing.smartbarHeight),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        SnyggIconButton(
            elementName = FlorisImeUi.ClipboardHeaderButton.elementName,
            onClick = {
                GifSearch.stopTyping()
                keyboardManager.activeState.imeUiMode = ImeUiMode.MEDIA
            },
            modifier = buttonSize,
        ) {
            SnyggIcon(imageVector = Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Back to GIFs")
        }
        SearchField(query, Modifier.weight(1f))
        SnyggIconButton(
            elementName = FlorisImeUi.ClipboardHeaderButton.elementName,
            onClick = {
                GifSearch.submit()
                keyboardManager.activeState.imeUiMode = ImeUiMode.MEDIA
            },
            modifier = buttonSize,
        ) {
            SnyggIcon(imageVector = Icons.Default.Search, contentDescription = "Search GIFs")
        }
    }
}

@Composable
private fun SearchField(query: String, modifier: Modifier) {
    val blink = rememberInfiniteTransition(label = "cursor").animateFloat(
        initialValue = 1f,
        targetValue = 0f,
        animationSpec = infiniteRepeatable(tween(durationMillis = 530), RepeatMode.Reverse),
        label = "cursor blink",
    )
    Row(
        modifier = modifier
            .fillMaxHeight()
            .padding(vertical = 6.dp)
            .clip(RoundedCornerShape(50))
            .background(Placeholder)
            .padding(horizontal = 14.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        if (query.isEmpty()) {
            Cursor(blink)
            Text("Search GIPHY", modifier = Modifier.padding(start = 4.dp).alpha(0.6f), maxLines = 1)
        } else {
            Text(query, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f, fill = false))
            Cursor(blink)
        }
    }
}

/** Reads [blink] while drawing, so the blink redraws this bar alone, not the whole field. */
@Composable
private fun Cursor(blink: State<Float>) {
    Box(
        Modifier
            .width(2.dp)
            .height(18.dp)
            .graphicsLayer { alpha = blink.value }
            .background(LocalContentColor.current),
    )
}
