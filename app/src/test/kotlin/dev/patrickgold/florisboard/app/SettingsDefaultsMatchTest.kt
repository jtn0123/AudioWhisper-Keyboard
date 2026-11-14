package dev.patrickgold.florisboard.app

import com.jtn0123.audiowhisper.rust.SettingsRules
import dev.patrickgold.jetpref.datastore.model.PreferenceData
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.random.Random

class SettingsDefaultsMatchTest {
    private val legacy = ReferenceSettingsModel().declaredPreferenceEntries.values.associateBy { it.key }
    private val native = FlorisPreferenceModelImpl().declaredPreferenceEntries.values.associateBy { it.key }
    private fun <T : Any> defaultRaw(p: PreferenceData<T>) = checkNotNull(p.serializer.serialize(p.default))
    private fun <T : Any> parsed(p: PreferenceData<T>, raw: String): Pair<Boolean,String?> = try {
        val value=p.serializer.deserialize(raw)
        true to value?.let { p.serializer.serialize(it) }
    } catch (_: Exception) { false to null }

    @Test fun everyDefaultAndStoredScalarBoundaryMatches() {
        assertEquals(143,legacy.size)
        assertEquals(legacy.keys,native.keys)
        assertEquals(legacy.keys,SettingsRules.catalogue.keys)
        for ((key,old) in legacy) {
            val new=native.getValue(key)
            assertEquals(old.type,new.type,key)
            assertEquals(defaultRaw(old),defaultRaw(new),key)
            val choices=(old.default as? Enum<*>)?.javaClass?.enumConstants?.map { it.name }.orEmpty()
            val numeric = when(old.default) {
                is Int -> listOf(Int.MIN_VALUE.toString(),Int.MAX_VALUE.toString(),"-1","0","1","+1","2147483648","-2147483649","١٢")
                is Long -> listOf(Long.MIN_VALUE.toString(),Long.MAX_VALUE.toString(),"-1","0","1","9223372036854775808")
                is Float -> listOf("NaN","Infinity","-Infinity","-0.0","0.0","1.4E-45","3.4028235E38","1e99","0x1p-2f")
                else -> emptyList()
            }
            val bounds=referenceSettingsBounds[key].orEmpty().flatMap { n -> listOf(n.toDouble()-1,n.toDouble(),n.toDouble()+1) }.map { if(old.default is Int) it.toInt().toString() else it.toString() }
            // Complex values have separate codec differential tests; ordinary strings include arbitrary UTF-16.
            if (key in setOf("emoji__history_data","smartbar__action_arrangement")) continue
            for (raw in (listOf(defaultRaw(old),"","true","false","TRUE","unknown"," ","\ud800")+choices+numeric+bounds).distinct()) {
                assertEquals(parsed(old,raw),parsed(new,raw),"$key scalar case")
            }
        }
    }
    @Test fun everySliderRangeStepAndClampMatchesFrozenDeclarations() {
        val random=Random(17)
        for ((key,range) in referenceSettingsBounds) {
            val (lo,hi,step)=range.map { it.toFloat() }
            assertEquals(lo,SettingsRules.minimum(key).toFloat(),key)
            assertEquals(hi,SettingsRules.maximum(key).toFloat(),key)
            assertEquals(step,SettingsRules.step(key).toFloat(),key)
            val cases=listOf(lo-1,lo,lo+step,hi-step,hi,hi+1,Float.NEGATIVE_INFINITY,Float.POSITIVE_INFINITY,Float.NaN,-0f)+List(200) { random.nextFloat()*10000-5000 }
            for(value in cases) assertEquals(value.coerceIn(lo,hi).toRawBits(),SettingsRules.clamp(key,value.toString()).toFloat().toRawBits(),key)
        }
    }
}
