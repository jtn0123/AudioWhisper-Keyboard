/*
 * Copyright (C) 2026 The AudioWhisper Keyboard Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#ifndef LATINIME_WORD_LISTENER_H
#define LATINIME_WORD_LISTENER_H

#include "defines.h"
#include "utils/int_array_view.h"

namespace latinime {

/**
 * Receives every word of a dictionary walk (DictionaryStructureWithBufferPolicy::iterateWords).
 */
class WordListener {
 public:
    // The word's code points; the view is only valid during the call.
    virtual void onVisitWord(const CodePointArrayView word) = 0;
    virtual ~WordListener() {};

 protected:
    WordListener() {}

 private:
    DISALLOW_COPY_AND_ASSIGN(WordListener);
};
} // namespace latinime
#endif /* LATINIME_WORD_LISTENER_H */
