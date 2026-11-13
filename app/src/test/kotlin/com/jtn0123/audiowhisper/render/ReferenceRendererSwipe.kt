/*
 * Copyright (C) 2021-2025 The FlorisBoard Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package com.jtn0123.audiowhisper.render

import com.jtn0123.audiowhisper.rust.RenderProtocol.*
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData

import com.jtn0123.audiowhisper.gif.GifSearchKeys
import dev.patrickgold.florisboard.ime.editor.OperationScope
import dev.patrickgold.florisboard.ime.editor.OperationUnit
import dev.patrickgold.florisboard.ime.text.gestures.SwipeAction
import dev.patrickgold.florisboard.ime.text.gestures.SwipeGesture
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import kotlin.math.abs

// Test-only frozen Compose swipe policy from 7c0b8183; editor/platform calls append trace packets.

internal fun ReferenceRendererTouch.handleSwipe(event: SwipeGesture.Event): Boolean {
    val pointer = pointerMap.findById(event.pointerId) ?: return false
    val initialKey = pointer.initialKey ?: return false
    val activeKey = pointer.activeKey


    return when (initialKey.computedData.code) {
        KeyCode.DELETE -> handleDeleteSwipe(event)
        KeyCode.SPACE, KeyCode.CJK_SPACE -> handleSpaceSwipe(event)
        else -> when {
            (initialKey.computedData.code == KeyCode.SHIFT && activeKey?.computedData?.code == KeyCode.SPACE ||
                initialKey.computedData.code == KeyCode.SHIFT && activeKey?.computedData?.code == KeyCode.CJK_SPACE) &&
                event.type == SwipeGesture.Type.TOUCH_MOVE -> handleSpaceSwipe(event)
            initialKey.computedData.code == KeyCode.SHIFT && activeKey?.computedData?.code != KeyCode.SHIFT &&
                event.type == SwipeGesture.Type.TOUCH_UP -> {
                activeKey?.let {
                    inputEventDispatcher.sendUp(popupUiController.getActiveKeyData(it) ?: it.computedDataOnDown)
                }
                inputEventDispatcher.cancelCode(KeyCode.SHIFT)
                emit(KEY_CANCEL_CODE, KeyCode.SHIFT, event.pointerId)
                true
            }
            initialKey.computedData.code > KeyCode.SPACE && !popupUiController.isShowingExtendedPopup -> when {
                !isGlideEnabled && !pointer.hasTriggeredGestureMove -> when (event.type) {
                    SwipeGesture.Type.TOUCH_UP -> {
                        val swipeAction = when (event.direction) {
                            SwipeGesture.Direction.UP -> prefs.gestures.swipeUp.get()
                            SwipeGesture.Direction.DOWN -> prefs.gestures.swipeDown.get()
                            SwipeGesture.Direction.LEFT -> prefs.gestures.swipeLeft.get()
                            SwipeGesture.Direction.RIGHT -> prefs.gestures.swipeRight.get()
                            else -> SwipeAction.NO_ACTION
                        }
                        if (swipeAction != SwipeAction.NO_ACTION) {
                            keyboardManager.executeSwipeAction(swipeAction)
                            true
                        } else {
                            false
                        }
                    }
                    else -> false
                }
                else -> false
            }
            else -> false
        }
    }
}

/** A delete-key swipe ended: delete what it selected. */
internal fun ReferenceRendererTouch.deleteSwipedSelection() {
    // AudioWhisper: the GIF search box deleted as the swipe went, and the app's selection must stay.
    if (keyboardManager.isTypingIntoGifSearch) return
    val selection = editorInstance.activeContent.selection
    if (selection.isSelectionMode) {
        editorInstance.deleteBackwards(OperationUnit.CHARACTERS)
    }
}

/** AudioWhisper: in the GIF search box a delete swipe shrinks the query as it goes; swiping back restores it. */
private fun ReferenceRendererTouch.deleteSwipeInGifSearch(event: SwipeGesture.Event): Boolean {
    if (GifSearchKeys.swipeDelete(prefs.gestures.deleteKeySwipeLeft.get(), event.absUnitCountX) == null) return false
    if (abs(event.relUnitCountX) > 0) {
        inputFeedbackController?.gestureMovingSwipe(TextKeyData.DELETE)
    }
    val starts = pointerMap.findById(event.pointerId)?.hasTriggeredGestureMove != true
    emit(if (prefs.gestures.deleteKeySwipeLeft.get() in listOf(SwipeAction.DELETE_WORDS_PRECISELY, SwipeAction.SELECT_WORDS_PRECISELY)) GIF_DELETE_WORDS else GIF_DELETE_CHARACTERS, event.absUnitCountX, if (starts) 1 else 0)
    return true
}

private fun ReferenceRendererTouch.handleDeleteSwipe(event: SwipeGesture.Event): Boolean {
    // AudioWhisper: the swipe's end goes through executeSwipeAction, which the GIF search gates.
    if (keyboardManager.isTypingIntoGifSearch && event.type == SwipeGesture.Type.TOUCH_MOVE) {
        return deleteSwipeInGifSearch(event)
    }
    if (editorInstance.activeInfo.isRawInputEditor) return false

    return when (event.type) {
        SwipeGesture.Type.TOUCH_MOVE -> when (prefs.gestures.deleteKeySwipeLeft.get()) {
            SwipeAction.DELETE_CHARACTERS_PRECISELY, SwipeAction.SELECT_CHARACTERS_PRECISELY -> {
                if (abs(event.relUnitCountX) > 0) {
                    inputFeedbackController?.gestureMovingSwipe(TextKeyData.DELETE)
                }
                val activeSelection = editorInstance.activeContent.selection
                if (activeSelection.isValid) {
                    if (!inputEventDispatcher.isPressed(KeyCode.SHIFT)) {
                        // Backward select
                        editorInstance.setSelectionSurrounding(
                            n = -event.absUnitCountX - 1,
                            unit = OperationUnit.CHARACTERS,
                            scope = OperationScope.BEFORE_CURSOR,
                        )
                    } else {
                        // Forward select
                        editorInstance.setSelectionSurrounding(
                            n = -event.absUnitCountX - 1,
                            unit = OperationUnit.CHARACTERS,
                            scope = OperationScope.AFTER_CURSOR,
                        )
                    }
                }
                true
            }
            SwipeAction.DELETE_WORDS_PRECISELY, SwipeAction.SELECT_WORDS_PRECISELY -> {
                if (abs(event.relUnitCountX) > 0) {
                    inputFeedbackController?.gestureMovingSwipe(TextKeyData.DELETE)
                }
                val activeSelection = editorInstance.activeContent.selection
                if (activeSelection.isValid) {
                    if (!inputEventDispatcher.isPressed(KeyCode.SHIFT)) {
                        // Backward select
                        editorInstance.setSelectionSurrounding(
                            n = -event.absUnitCountX / 2 - 1,
                            unit = OperationUnit.WORDS,
                            scope = OperationScope.BEFORE_CURSOR,
                        )
                    } else {
                        // Forward select
                        editorInstance.setSelectionSurrounding(
                            n = -event.absUnitCountX / 2 - 1,
                            unit = OperationUnit.WORDS,
                            scope = OperationScope.AFTER_CURSOR,
                        )
                    }
                }
                true
            }
            else -> false
        }
        SwipeGesture.Type.TOUCH_UP -> {
            if (event.direction == SwipeGesture.Direction.LEFT &&
                prefs.gestures.deleteKeySwipeLeft.get() == SwipeAction.DELETE_WORD
            ) {
                keyboardManager.executeSwipeAction(prefs.gestures.deleteKeySwipeLeft.get())
                true
            } else {
                false
            }
        }
    }
}

private fun ReferenceRendererTouch.handleSpaceSwipe(event: SwipeGesture.Event): Boolean {
    val pointer = pointerMap.findById(event.pointerId) ?: return false
    // AudioWhisper: the GIF search box has no cursor to move, and the app's must stay put.
    // Taking the move also keeps the swipe from typing a space; its end is gated as a key.
    if (keyboardManager.isTypingIntoGifSearch && event.type == SwipeGesture.Type.TOUCH_MOVE) return true

    return when (event.type) {
        SwipeGesture.Type.TOUCH_MOVE -> when (event.direction) {
            SwipeGesture.Direction.LEFT -> {
                val action = prefs.gestures.spaceBarSwipeLeft.get()
                if (action == SwipeAction.MOVE_CURSOR_LEFT) {
                    abs(event.relUnitCountX).let {
                        val count = if (!pointer.hasTriggeredGestureMove) it - 1 else it
                        if (count > 0) {
                            inputFeedbackController?.gestureMovingSwipe(TextKeyData.SPACE)
                            if (!pointer.hasTriggeredMassSelection) {
                                pointer.hasTriggeredMassSelection = true
                                editorInstance.massSelection.begin()
                            }
                            keyboardManager.handleArrow(KeyCode.ARROW_LEFT, count)
                        }
                    }
                    true
                } else {
                    action != SwipeAction.NO_ACTION
                }
            }
            SwipeGesture.Direction.RIGHT -> {
                val action = prefs.gestures.spaceBarSwipeRight.get()
                if (action == SwipeAction.MOVE_CURSOR_RIGHT) {
                    abs(event.relUnitCountX).let {
                        val count = if (!pointer.hasTriggeredGestureMove) it - 1 else it
                        if (count > 0) {
                            inputFeedbackController?.gestureMovingSwipe(TextKeyData.SPACE)
                            if (!pointer.hasTriggeredMassSelection) {
                                pointer.hasTriggeredMassSelection = true
                                editorInstance.massSelection.begin()
                            }
                            keyboardManager.handleArrow(KeyCode.ARROW_RIGHT, count)
                        }
                    }
                    true
                } else {
                    action != SwipeAction.NO_ACTION
                }
            }
            else -> false
        }
        SwipeGesture.Type.TOUCH_UP -> when (event.direction) {
            SwipeGesture.Direction.LEFT -> {
                prefs.gestures.spaceBarSwipeLeft.get().let {
                    when {
                        it == SwipeAction.NO_ACTION -> {
                            false
                        }
                        it != SwipeAction.MOVE_CURSOR_LEFT -> {
                            keyboardManager.executeSwipeAction(it)
                            true
                        }
                        else -> {
                            false
                        }
                    }
                }
            }
            SwipeGesture.Direction.RIGHT -> {
                prefs.gestures.spaceBarSwipeRight.get().let {
                    when {
                        it == SwipeAction.NO_ACTION -> {
                            false
                        }
                        it != SwipeAction.MOVE_CURSOR_RIGHT -> {
                            keyboardManager.executeSwipeAction(it)
                            true
                        }
                        else -> {
                            false
                        }
                    }
                }
            }
            else -> {
                if (event.absUnitCountY < -6) {
                    keyboardManager.executeSwipeAction(prefs.gestures.spaceBarSwipeUp.get())
                    true
                } else {
                    false
                }
            }
        }
    }
}
