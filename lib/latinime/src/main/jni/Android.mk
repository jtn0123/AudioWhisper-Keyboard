# Copyright (C) 2011 The Android Open Source Project
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#      http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# AudioWhisper Keyboard: the AOSP LatinIME dictionary engine (via HeliBoard
# v4.2's copy), built as libjni_latinime.so. Same sources and warning flags as
# AOSP, minus its -Wno-* switches, so every warning the engine produces is
# visible in the build log.

LOCAL_PATH := $(call my-dir)

######################################
include $(CLEAR_VARS)

LATIN_IME_SRC_DIR := src

LOCAL_C_INCLUDES += $(LOCAL_PATH)/$(LATIN_IME_SRC_DIR)

LOCAL_CFLAGS += -Wall -Wextra -Weffc++ -Wformat=2 -Wcast-qual -Wcast-align \
    -Wwrite-strings -Wfloat-equal -Wpointer-arith -Winit-self -Wredundant-decls \
    -Woverloaded-virtual -Wsign-promo

include $(LOCAL_PATH)/NativeFileList.mk

LOCAL_SRC_FILES := \
    $(LATIN_IME_JNI_SRC_FILES) \
    $(addprefix $(LATIN_IME_SRC_DIR)/, $(LATIN_IME_CORE_SRC_FILES))

LOCAL_MODULE := libjni_latinime_common_static

include $(BUILD_STATIC_LIBRARY)
######################################
include $(CLEAR_VARS)

# All code in LOCAL_WHOLE_STATIC_LIBRARIES will be built into this shared library.
LOCAL_WHOLE_STATIC_LIBRARIES := libjni_latinime_common_static

LOCAL_MODULE := libjni_latinime
LOCAL_LDLIBS += -ldl

include $(BUILD_SHARED_LIBRARY)
#################### Clean up the tmp vars
include $(LOCAL_PATH)/CleanupNativeFileList.mk
