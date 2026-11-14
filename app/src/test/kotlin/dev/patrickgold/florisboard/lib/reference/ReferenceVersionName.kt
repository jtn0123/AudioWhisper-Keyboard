package dev.patrickgold.florisboard.lib.reference

data class ReferenceVersionName(
    val major: Int,
    val minor: Int,
    val patch: Int,
    val extraName: String? = null,
    val extraValue: Int? = null
) {
    companion object {
        val DEFAULT: ReferenceVersionName = ReferenceVersionName(0, 0, 0)
        const val DEFAULT_RAW: String = "0.0.0"

        fun fromString(raw: String): ReferenceVersionName? {
            if (raw.matches("""[0-9]+[.][0-9]+[.][0-9]+""".toRegex())) {
                val list = raw.split(".").map { it.toInt() }
                if (list.size == 3) {
                    return ReferenceVersionName(list[0], list[1], list[2])
                }
            } else if (raw.matches("""[0-9]+[.][0-9]+[.][0-9]+[-][0-9]+""".toRegex())) {
                val list = raw.split(".").map { it.toInt() }
                if (list.size == 4) {
                    return ReferenceVersionName(list[0], list[1], list[2], null, list[3])
                }
            } else if (raw.matches("""[0-9]+[.][0-9]+[.][0-9]+[-][a-zA-Z]+""".toRegex())) {
                val list = raw.split(".")
                if (list.size == 4) {
                    return ReferenceVersionName(
                        list[0].toInt(), list[1].toInt(), list[2].toInt(),
                        list[3], null
                    )
                }
            } else if (raw.matches("""[0-9]+[.][0-9]+[.][0-9]+[-][a-zA-Z]+[0-9]+""".toRegex())) {
                val list = raw.split(".")
                if (list.size == 4) {
                    val extraName = list[3].split("""[0-9]+""".toRegex())[0]
                    val extraValue = list[3].split("""[a-zA-Z]+""".toRegex())[1].toInt()
                    return ReferenceVersionName(
                        list[0].toInt(), list[1].toInt(), list[2].toInt(),
                        extraName, extraValue
                    )
                }
            }
            return null
        }
    }

    override fun toString(): String {
        val mmp = "$major.$minor.$patch"
        return if (extraName != null || extraValue != null) {
            val extraName = extraName ?: ""
            val extraValue = extraValue?.toString() ?: ""
            "$mmp.$extraName$extraValue"
        } else {
            mmp
        }
    }

    operator fun compareTo(other: ReferenceVersionName): Int {
        if (major != other.major) {
            return major.compareTo(other.major)
        } else if (minor != other.minor) {
            return minor.compareTo(other.minor)
        } else if (patch != other.patch) {
            return patch.compareTo(other.patch)
        } else if (extraValue != null && other.extraValue != null) {
            if (extraValue != other.extraValue) {
                return extraValue.compareTo(other.extraValue)
            }
        }
        return 0
    }
}
