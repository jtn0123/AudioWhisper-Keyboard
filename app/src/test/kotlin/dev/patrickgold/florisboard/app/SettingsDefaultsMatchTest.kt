package dev.patrickgold.florisboard.app

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.SettingsRules
import dev.patrickgold.jetpref.datastore.model.PreferenceData
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.random.Random

/** The settings a slider sets, in the order of the frozen Compose declarations (33b8db50). */
internal val sliderKeys = listOf("glide__trail_fade_duration", "glide__preview_refresh_delay",
    "gestures__swipe_velocity_threshold", "gestures__swipe_distance_threshold", "keyboard__font_size_multiplier_portrait",
    "keyboard__font_size_multiplier_landscape", "keyboard__one_handed_mode_scale_factor", "keyboard__height_factor_portrait",
    "keyboard__height_factor_landscape", "keyboard__key_spacing_vertical", "keyboard__key_spacing_horizontal",
    "keyboard__bottom_offset_portrait", "keyboard__bottom_offset_landscape", "keyboard__long_press_delay",
    "input_feedback__audio_volume", "input_feedback__haptic_vibration_duration", "input_feedback__haptic_vibration_strength",
    "emoji__history_pinned_max_size", "emoji__history_recent_max_size", "emoji__suggestion_query_min_length",
    "emoji__suggestion_candidate_max_count", "clipboard__suggestion_timeout", "clipboard__history_num_grid_columns_portrait",
    "clipboard__history_num_grid_columns_landscape", "clipboard__history_auto_clean_old_after",
    "clipboard__history_auto_clean_sensitive_after", "clipboard__history_size_limit")

class SettingsDefaultsMatchTest {
    private val native = FlorisPreferenceModelImpl().declaredPreferenceEntries.values.associateBy { it.key }
    private fun <T : Any> defaultRaw(p: PreferenceData<T>) = checkNotNull(p.serializer.serialize(p.default))
    private fun <T : Any> parsed(p: PreferenceData<T>, raw: String): Pair<Boolean,String?> = try {
        val value=p.serializer.deserialize(raw)
        true to value?.let { p.serializer.serialize(it) }
    } catch (_: Exception) { false to null }
    /** A slider's minimum, maximum and step, as the catalogue states them. */
    private fun bounds(key: String) = listOf(SettingsRules.minimum(key),SettingsRules.maximum(key),SettingsRules.step(key))

    @Test fun everyDefaultAndStoredScalarBoundaryMatches() {
        assertEquals(145,native.size)
        assertEquals(native.keys,SettingsRules.catalogue.keys)
        val tape=GoldenTape("settings-defaults")
        tape.answer("keys",native.keys.sorted())
        for (key in native.keys.sorted()) {
            val p=native.getValue(key)
            tape.answer("$key type",p.type.id)
            tape.answer("$key default",defaultRaw(p))
            val choices=(p.default as? Enum<*>)?.javaClass?.enumConstants?.map { it.name }.orEmpty()
            val numeric = when(p.default) {
                is Int -> listOf(Int.MIN_VALUE.toString(),Int.MAX_VALUE.toString(),"-1","0","1","+1","2147483648","-2147483649","١٢")
                is Long -> listOf(Long.MIN_VALUE.toString(),Long.MAX_VALUE.toString(),"-1","0","1","9223372036854775808")
                is Float -> listOf("NaN","Infinity","-Infinity","-0.0","0.0","1.4E-45","3.4028235E38","1e99","0x1p-2f")
                else -> emptyList()
            }
            val edges=(if (key in sliderKeys) bounds(key) else emptyList()).flatMap { n -> listOf(n.toDouble()-1,n.toDouble(),n.toDouble()+1) }
                .map { if(p.default is Int) it.toInt().toString() else it.toString() }
            // Complex values have separate codec tests; ordinary strings include arbitrary UTF-16.
            if (key in setOf("emoji__history_data","smartbar__action_arrangement")) continue
            for (raw in (listOf(defaultRaw(p),"","true","false","TRUE","unknown"," ","\ud800")+choices+numeric+edges).distinct())
                tape.answer("$key $raw",parsed(p,raw))
        }
        tape.finish()
    }
    @Test fun everySliderRangeStepAndClampMatchesFrozenDeclarations() {
        val random=Random(17)
        val tape=GoldenTape("settings-sliders")
        for (key in sliderKeys) {
            val (lo,hi,step)=bounds(key).map { it.toFloat() }
            tape.answer("$key bounds",listOf(lo,hi,step))
            val cases=listOf(lo-1,lo,lo+step,hi-step,hi,hi+1,Float.NEGATIVE_INFINITY,Float.POSITIVE_INFINITY,Float.NaN,-0f)+List(200) { random.nextFloat()*10000-5000 }
            for(value in cases) tape.answer("$key $value",SettingsRules.clamp(key,value.toString()).toFloat().toRawBits())
        }
        tape.finish()
    }
}
