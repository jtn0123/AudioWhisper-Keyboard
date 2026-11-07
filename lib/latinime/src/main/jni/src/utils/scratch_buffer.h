/*
 * SPDX-License-Identifier: Apache-2.0
 */

#ifndef LATINIME_SCRATCH_BUFFER_H
#define LATINIME_SCRATCH_BUFFER_H

#include <cstddef>
#include <vector>

#include "defines.h"

namespace latinime {

/**
 * A scratch array whose length is only known at run time: the standard C++ replacement for a
 * variable-length array. A length up to InlineCapacity uses storage inside the object, so the
 * common case lives on the stack and allocates nothing; a longer one falls back to the heap.
 * As with a variable-length array, the inline storage is left uninitialized.
 */
template <typename T, size_t InlineCapacity>
class ScratchBuffer {
 public:
    explicit ScratchBuffer(const int length)
            : mHeapStorage(fitsInline(length) ? 0 : static_cast<size_t>(length)),
              mData(fitsInline(length) ? mInlineStorage : mHeapStorage.data()) {}

    T *data() { return mData; }

 private:
    DISALLOW_IMPLICIT_CONSTRUCTORS(ScratchBuffer);

    static bool fitsInline(const int length) {
        return length <= static_cast<int>(InlineCapacity);
    }

    T mInlineStorage[InlineCapacity];
    std::vector<T> mHeapStorage;
    T *const mData;
};
} // namespace latinime
#endif // LATINIME_SCRATCH_BUFFER_H
