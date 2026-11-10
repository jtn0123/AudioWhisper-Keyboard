/*
 * A JNI environment just big enough to run LatinIME's real JNI glue outside a JVM, so the Rust
 * port can be compared with the C++ it replaces in plain `cargo test`.
 *
 * JNI_OnLoad (jni/jni_common.cpp) registers each native method through RegisterNatives; this
 * file records them by class and name, and the aw_oracle_* functions below call them exactly as
 * the Java glue in lib/latinime does, with the same array sizes. Java arrays are views of the
 * caller's memory, so what the C++ writes lands in the caller's buffers.
 *
 * Any JNI function the glue reaches that isn't here aborts with its slot number, rather than
 * running on with a wrong answer.
 */

#include <jni.h>

#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <map>
#include <string>
#include <utility>
#include <vector>

#include "utils/time_keeper.h"

namespace {

enum class Kind { IntArray, FloatArray, BooleanArray, ObjectArray, String, Class };

/** What a jobject points to here. Arrays don't own their elements; a String owns its text. */
struct Object {
    Kind kind;
    jsize length;
    void *elements;
    std::u16string text;
    std::string name;
};

[[noreturn]] void fail(const char *what) {
    std::fprintf(stderr, "latin-oracle fake JNI: %s\n", what);
    std::abort();
}

Object *object(jobject handle, Kind kind) {
    if (handle == nullptr) fail("null object");
    Object *const it = reinterpret_cast<Object *>(handle);
    if (it->kind != kind) fail("object of the wrong kind");
    return it;
}

void checkRegion(const Object *array, jsize start, jsize length) {
    if (start < 0 || length < 0 || start > array->length - length) fail("array index out of bounds");
}

template <typename T, typename ArrayType, Kind kind>
void getRegion(JNIEnv *, ArrayType array, jsize start, jsize length, T *buffer) {
    Object *const it = object(array, kind);
    checkRegion(it, start, length);
    std::memcpy(buffer, static_cast<T *>(it->elements) + start, sizeof(T) * length);
}

template <typename T, typename ArrayType, Kind kind>
void setRegion(JNIEnv *, ArrayType array, jsize start, jsize length, const T *buffer) {
    Object *const it = object(array, kind);
    checkRegion(it, start, length);
    std::memcpy(static_cast<T *>(it->elements) + start, buffer, sizeof(T) * length);
}

jsize getArrayLength(JNIEnv *, jarray array) {
    Object *const it = reinterpret_cast<Object *>(array);
    if (it == nullptr || it->kind == Kind::String || it->kind == Kind::Class) fail("not an array");
    return it->length;
}

jobject getObjectArrayElement(JNIEnv *, jobjectArray array, jsize index) {
    Object *const it = object(array, Kind::ObjectArray);
    checkRegion(it, index, 1);
    return static_cast<jobject *>(it->elements)[index];
}

/** The strings passed in here are ASCII, where modified UTF-8 and UTF-16 count alike. */
jsize getStringLength(JNIEnv *, jstring string) {
    return static_cast<jsize>(object(string, Kind::String)->text.size());
}

void getStringUtfRegion(JNIEnv *, jstring string, jsize start, jsize length, char *buffer) {
    Object *const it = object(string, Kind::String);
    if (start < 0 || length < 0 || static_cast<size_t>(start + length) > it->text.size()) {
        fail("string index out of bounds");
    }
    for (jsize i = 0; i < length; ++i) buffer[i] = static_cast<char>(it->text[start + i]);
}

jstring newString(JNIEnv *, const jchar *chars, jsize length) {
    Object *const it = new Object{Kind::String, length, nullptr, {}, {}};
    it->text.assign(reinterpret_cast<const char16_t *>(chars), length);
    return reinterpret_cast<jstring>(it);
}

std::vector<Object *> &classes() {
    static std::vector<Object *> all;
    return all;
}

jclass findClass(JNIEnv *, const char *name) {
    for (Object *const it : classes()) {
        if (it->name == name) return reinterpret_cast<jclass>(it);
    }
    classes().push_back(new Object{Kind::Class, 0, nullptr, {}, name});
    return reinterpret_cast<jclass>(classes().back());
}

std::map<std::string, void *> &natives() {
    static std::map<std::string, void *> all;
    return all;
}

jint registerNatives(JNIEnv *, jclass clazz, const JNINativeMethod *methods, jint count) {
    const std::string &className = object(clazz, Kind::Class)->name;
    for (jint i = 0; i < count; ++i) {
        natives()[className + "." + methods[i].name] = methods[i].fnPtr;
    }
    return JNI_OK;
}

void deleteLocalRef(JNIEnv *, jobject) {}

/**
 * LogUtils::logToJava looks for android.util.Log.i, and gives up quietly when it isn't there:
 * logging is the only thing reaching for static methods.
 */
jmethodID getStaticMethodId(JNIEnv *, jclass, const char *, const char *) { return nullptr; }

void exceptionClear(JNIEnv *) {}

jint getVersion(JNIEnv *) { return JNI_VERSION_1_6; }

/** Which slot an unimplemented call came through, found by the address the table holds. */
template <size_t slot>
void unimplemented() {
    std::fprintf(stderr, "latin-oracle fake JNI: function slot %zu is not implemented\n", slot);
    std::abort();
}

constexpr size_t SLOTS = sizeof(JNINativeInterface) / sizeof(void *);

template <size_t... slots>
void fillTraps(void **table, std::index_sequence<slots...>) {
    ((table[slots] = reinterpret_cast<void *>(&unimplemented<slots>)), ...);
}

JNINativeInterface makeTable() {
    JNINativeInterface table;
    fillTraps(reinterpret_cast<void **>(&table), std::make_index_sequence<SLOTS>());
    table.GetVersion = getVersion;
    table.FindClass = findClass;
    table.RegisterNatives = registerNatives;
    table.DeleteLocalRef = deleteLocalRef;
    table.GetStaticMethodID = getStaticMethodId;
    table.ExceptionClear = exceptionClear;
    table.GetArrayLength = getArrayLength;
    table.GetObjectArrayElement = getObjectArrayElement;
    table.GetStringLength = getStringLength;
    table.GetStringUTFLength = getStringLength;
    table.GetStringUTFRegion = getStringUtfRegion;
    table.NewString = newString;
    table.GetIntArrayRegion = getRegion<jint, jintArray, Kind::IntArray>;
    table.SetIntArrayRegion = setRegion<jint, jintArray, Kind::IntArray>;
    table.GetFloatArrayRegion = getRegion<jfloat, jfloatArray, Kind::FloatArray>;
    table.SetFloatArrayRegion = setRegion<jfloat, jfloatArray, Kind::FloatArray>;
    table.GetBooleanArrayRegion = getRegion<jboolean, jbooleanArray, Kind::BooleanArray>;
    table.SetBooleanArrayRegion = setRegion<jboolean, jbooleanArray, Kind::BooleanArray>;
    return table;
}

const JNINativeInterface TABLE = makeTable();
JNIEnv ENV{&TABLE};

jint getEnv(JavaVM *, void **env, jint) {
    *env = &ENV;
    return JNI_OK;
}

JNIInvokeInterface makeVmTable() {
    JNIInvokeInterface table;
    std::memset(&table, 0, sizeof(table));
    table.GetEnv = getEnv;
    return table;
}

const JNIInvokeInterface VM_TABLE = makeVmTable();
JavaVM VM{&VM_TABLE};

/** The native registered as class.name, cast to the type it was registered with. */
template <typename Function>
Function native(const char *qualifiedName) {
    const auto it = natives().find(qualifiedName);
    if (it == natives().end()) fail(qualifiedName);
    return reinterpret_cast<Function>(it->second);
}

/** A Java array over the caller's memory, alive for one call. */
template <Kind kind>
struct Array {
    Object object;
    Array(const void *elements, jsize length)
            : object{kind, length, const_cast<void *>(elements), {}, {}} {}
    jobject handle() { return reinterpret_cast<jobject>(&object); }
};

using IntArray = Array<Kind::IntArray>;

struct String {
    Object object;
    explicit String(const char *ascii) : object{Kind::String, 0, nullptr, {}, {}} {
        for (const char *c = ascii; *c != '\0'; ++c) object.text.push_back(static_cast<char16_t>(*c));
        object.length = static_cast<jsize>(object.text.size());
    }
    jstring handle() { return reinterpret_cast<jstring>(&object); }
};

/**
 * The previous words as Java's int[][]: `count` slots of an array `slots` long, a negative
 * length leaving that slot null. `flat` holds the words one after another.
 */
struct PreviousWords {
    std::vector<IntArray> words;
    std::vector<jobject> handles;
    Array<Kind::ObjectArray> array;

    PreviousWords(const jint *flat, const jint *lengths, jsize slots)
            : words(), handles(slots, nullptr), array(nullptr, slots) {
        words.reserve(slots);
        jsize start = 0;
        for (jsize i = 0; i < slots; ++i) {
            if (lengths[i] < 0) continue;
            words.emplace_back(flat + start, lengths[i]);
            handles[i] = words.back().handle();
            start += lengths[i];
        }
        array.object.elements = handles.data();
    }
};

const char *const DICTIONARY = "com/android/inputmethod/latin/BinaryDictionary.";
const char *const SESSION = "com/android/inputmethod/latin/DicTraverseSession.";
const char *const PROXIMITY = "com/android/inputmethod/keyboard/ProximityInfo.";

std::string name(const char *clazz, const char *method) { return std::string(clazz) + method; }

} // namespace

extern "C" jint JNI_OnLoad(JavaVM *vm, void *reserved);

extern "C" {

int32_t aw_oracle_load() { return JNI_OnLoad(&VM, nullptr); }

/**
 * TimeKeeper held at the current time, in test mode, so Dictionary::getSuggestions's
 * setCurrentTime only reads its statics while searches run on several threads; and released,
 * back on the clock as a lone getSuggestions would leave it. Called with no other call running.
 */
void aw_oracle_hold_time() {
    latinime::TimeKeeper::setCurrentTime();
    latinime::TimeKeeper::startTestModeWithForceCurrentTime(
            latinime::TimeKeeper::peekCurrentTime());
}

void aw_oracle_release_time() {
    latinime::TimeKeeper::stopTestMode();
    latinime::TimeKeeper::setCurrentTime();
}

int64_t aw_oracle_open(const char *path, int64_t offset, int64_t size) {
    String source(path);
    return native<jlong (*)(JNIEnv *, jclass, jstring, jlong, jlong, jboolean)>(
            name(DICTIONARY, "openNative").c_str())(&ENV, nullptr, source.handle(), offset, size,
            JNI_FALSE);
}

void aw_oracle_close(int64_t dictionary) {
    native<void (*)(JNIEnv *, jclass, jlong)>(name(DICTIONARY, "closeNative").c_str())(
            &ENV, nullptr, dictionary);
}

int32_t aw_oracle_probability(int64_t dictionary, const int32_t *word, int32_t length) {
    IntArray array(word, length);
    return native<jint (*)(JNIEnv *, jclass, jlong, jintArray)>(
            name(DICTIONARY, "getProbabilityNative").c_str())(&ENV, nullptr, dictionary,
            static_cast<jintArray>(array.handle()));
}

int32_t aw_oracle_ngram_probability(int64_t dictionary, const int32_t *previousFlat,
        const int32_t *previousLengths, const uint8_t *beginningOfSentence, int32_t slots,
        const int32_t *word, int32_t length) {
    PreviousWords previous(previousFlat, previousLengths, slots);
    Array<Kind::BooleanArray> starts(beginningOfSentence, slots);
    IntArray target(word, length);
    return native<jint (*)(JNIEnv *, jclass, jlong, jobjectArray, jbooleanArray, jintArray)>(
            name(DICTIONARY, "getNgramProbabilityNative").c_str())(&ENV, nullptr, dictionary,
            static_cast<jobjectArray>(previous.array.handle()),
            static_cast<jbooleanArray>(starts.handle()), static_cast<jintArray>(target.handle()));
}

/** The words, each followed by '\n', as UTF-16, handed to `sink` before they are freed. */
void aw_oracle_words(int64_t dictionary, void (*sink)(void *, const uint16_t *, size_t),
        void *context) {
    jstring words = native<jstring (*)(JNIEnv *, jclass, jlong)>(
            name(DICTIONARY, "getWordsNative").c_str())(&ENV, nullptr, dictionary);
    if (words == nullptr) return;
    Object *const it = object(words, Kind::String);
    sink(context, reinterpret_cast<const uint16_t *>(it->text.data()), it->text.size());
    delete it;
}

int64_t aw_oracle_proximity(int32_t width, int32_t height, int32_t gridWidth, int32_t gridHeight,
        int32_t keyWidth, int32_t keyHeight, const int32_t *grid, int32_t gridLength,
        int32_t keyCount, const int32_t *xs, const int32_t *ys, const int32_t *widths,
        const int32_t *heights, const int32_t *codes) {
    IntArray proximity(grid, gridLength), x(xs, keyCount), y(ys, keyCount),
            w(widths, keyCount), h(heights, keyCount), c(codes, keyCount);
    return native<jlong (*)(JNIEnv *, jclass, jint, jint, jint, jint, jint, jint, jintArray, jint,
            jintArray, jintArray, jintArray, jintArray, jintArray, jfloatArray, jfloatArray,
            jfloatArray)>(name(PROXIMITY, "setProximityInfoNative").c_str())(&ENV, nullptr,
            width, height, gridWidth, gridHeight, keyWidth, keyHeight,
            static_cast<jintArray>(proximity.handle()), keyCount,
            static_cast<jintArray>(x.handle()), static_cast<jintArray>(y.handle()),
            static_cast<jintArray>(w.handle()), static_cast<jintArray>(h.handle()),
            static_cast<jintArray>(c.handle()), nullptr, nullptr, nullptr);
}

void aw_oracle_proximity_release(int64_t proximity) {
    native<void (*)(JNIEnv *, jclass, jlong)>(name(PROXIMITY, "releaseProximityInfoNative").c_str())(
            &ENV, nullptr, proximity);
}

/** A session set up as DicTraverseSession's constructor does: no previous word. */
int64_t aw_oracle_session(const char *locale, int64_t dictionary, int64_t dictionarySize) {
    String localeString(locale);
    const jlong session = native<jlong (*)(JNIEnv *, jclass, jstring, jlong)>(
            name(SESSION, "setDicTraverseSessionNative").c_str())(&ENV, nullptr,
            localeString.handle(), dictionarySize);
    native<void (*)(JNIEnv *, jclass, jlong, jlong, jintArray, jint)>(
            name(SESSION, "initDicTraverseSessionNative").c_str())(&ENV, nullptr, session,
            dictionary, nullptr, 0);
    return session;
}

void aw_oracle_session_release(int64_t session) {
    native<void (*)(JNIEnv *, jclass, jlong)>(name(SESSION, "releaseDicTraverseSessionNative").c_str())(
            &ENV, nullptr, session);
}

/**
 * BinaryDictionary.getSuggestions: every array is as long as Java's (`inputCapacity` for the
 * input arrays, `previousSlots` for the previous words, `options` for the options), and the
 * outputs are written in place.
 */
void aw_oracle_suggest(int64_t dictionary, int64_t proximity, int64_t session,
        const int32_t *xs, const int32_t *ys, const int32_t *times, const int32_t *pointerIds,
        const int32_t *codes, int32_t inputCapacity, int32_t inputSize,
        const int32_t *options, int32_t optionCount,
        const int32_t *previousFlat, const int32_t *previousLengths,
        const uint8_t *beginningOfSentence, int32_t previousSlots, int32_t previousCount,
        int32_t *outCount, int32_t *outCodePoints, int32_t codePointCapacity, int32_t *outScores,
        int32_t *outSpaceIndices, int32_t *outTypes, int32_t resultCapacity,
        int32_t *outAutoCommit, float *inOutWeight) {
    IntArray x(xs, inputCapacity), y(ys, inputCapacity), t(times, inputCapacity),
            p(pointerIds, inputCapacity), c(codes, inputCapacity), o(options, optionCount);
    PreviousWords previous(previousFlat, previousLengths, previousSlots);
    Array<Kind::BooleanArray> starts(beginningOfSentence, previousSlots);
    IntArray count(outCount, 1), codePoints(outCodePoints, codePointCapacity),
            scores(outScores, resultCapacity), spaces(outSpaceIndices, resultCapacity),
            types(outTypes, resultCapacity), autoCommit(outAutoCommit, 1);
    Array<Kind::FloatArray> weight(inOutWeight, 1);
    native<void (*)(JNIEnv *, jclass, jlong, jlong, jlong, jintArray, jintArray, jintArray,
            jintArray, jintArray, jint, jintArray, jobjectArray, jbooleanArray, jint, jintArray,
            jintArray, jintArray, jintArray, jintArray, jintArray, jfloatArray)>(
            name(DICTIONARY, "getSuggestionsNative").c_str())(&ENV, nullptr, dictionary,
            proximity, session, static_cast<jintArray>(x.handle()),
            static_cast<jintArray>(y.handle()), static_cast<jintArray>(t.handle()),
            static_cast<jintArray>(p.handle()), static_cast<jintArray>(c.handle()), inputSize,
            static_cast<jintArray>(o.handle()), static_cast<jobjectArray>(previous.array.handle()),
            static_cast<jbooleanArray>(starts.handle()), previousCount,
            static_cast<jintArray>(count.handle()), static_cast<jintArray>(codePoints.handle()),
            static_cast<jintArray>(scores.handle()), static_cast<jintArray>(spaces.handle()),
            static_cast<jintArray>(types.handle()), static_cast<jintArray>(autoCommit.handle()),
            static_cast<jfloatArray>(weight.handle()));
}

} // extern "C"
