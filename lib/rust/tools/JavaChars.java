// Writes one of crates/chars/src/tables/*.rs: how the platform it runs on answers Kotlin's Char
// functions, the regex \s, Java's code point functions and the per-character parts of its String
// case mapping, for every code point. The answers differ between a computer's JVM and Android,
// whose java.lang.Character, regex, Normalizer and String case mapping come from ICU, so it is run
// on each; tools/generate-char-tables.sh does both.
//
// The flag bits must match those in crates/chars/src/lib.rs. Each flag is the Character call that
// the Kotlin function of the same name makes (kotlin-stdlib's CharJVM.kt), except CASED and
// CASE_IGNORABLE, Unicode properties that Android's lower-casing of Σ consults (see
// crates/chars/src/case.rs).
import java.text.Normalizer;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.function.IntUnaryOperator;
import java.util.function.UnaryOperator;
import java.util.regex.Pattern;

public class JavaChars {
    static final int LETTER = 1;
    static final int DIGIT = 2;
    static final int LETTER_OR_DIGIT = 4;
    static final int UPPER = 8;
    static final int LOWER = 16;
    static final int SPACE = 32;
    static final int REGEX_SPACE = 64;
    static final int TITLE = 128;
    static final int CASED = 256;
    static final int CASE_IGNORABLE = 512;

    static final Pattern REGEX_S = Pattern.compile("\\s");

    /** U+10000, which has no case and isn't case-ignorable: a high surrogate that changes nothing. */
    static final String UNCASED_SUPPLEMENTARY = "𐀀";

    static final char CAPITAL_SIGMA = 'Σ';
    static final int HANGUL_FIRST = 0xAC00;
    static final int HANGUL_LAST = 0xD7A3;

    /** Android's ICU, which answers Cased and Case_Ignorable; null on the JVM. */
    static final java.lang.reflect.Method ICU_HAS_PROPERTY = icuMethod();
    static final int ICU_CASED = icuProperty("CASED");
    static final int ICU_CASE_IGNORABLE = icuProperty("CASE_IGNORABLE");

    static java.lang.reflect.Method icuMethod() {
        try {
            return Class.forName("android.icu.lang.UCharacter").getMethod("hasBinaryProperty", int.class, int.class);
        } catch (ReflectiveOperationException notAndroid) {
            return null;
        }
    }

    static int icuProperty(String name) {
        try {
            return Class.forName("android.icu.lang.UProperty").getField(name).getInt(null);
        } catch (ReflectiveOperationException notAndroid) {
            return -1;
        }
    }

    static boolean isAndroid() {
        return ICU_HAS_PROPERTY != null;
    }

    static boolean icu(int cp, int property) {
        try {
            return (Boolean) ICU_HAS_PROPERTY.invoke(null, cp, property);
        } catch (ReflectiveOperationException e) {
            throw new IllegalStateException(e);
        }
    }

    /** Unicode's Cased: Lowercase, Uppercase or Lt. The JVM's isLowerCase and isUpperCase include Other_*. */
    static boolean isCased(int cp) {
        if (isAndroid()) return icu(cp, ICU_CASED);
        return Character.isLowerCase(cp) || Character.isUpperCase(cp) || Character.isTitleCase(cp);
    }

    /**
     * Unicode's Case_Ignorable: Mn, Me, Cf, Lm or Sk, or Word_Break MidLetter, MidNumLet or
     * Single_Quote (WordBreakProperty.txt, Unicode 16). The JVM has no call for it.
     */
    static boolean isCaseIgnorable(int cp) {
        if (isAndroid()) return icu(cp, ICU_CASE_IGNORABLE);
        int type = Character.getType(cp);
        if (type == Character.NON_SPACING_MARK || type == Character.ENCLOSING_MARK || type == Character.FORMAT
                || type == Character.MODIFIER_LETTER || type == Character.MODIFIER_SYMBOL) {
            return true;
        }
        return WORD_BREAK_MID.indexOf(cp) >= 0;
    }

    /** Word_Break Single_Quote, MidNumLet and MidLetter, all in the BMP. */
    static final String WORD_BREAK_MID = "'"
            + ".‘’․﹒＇．"
            + ":··՟״‧︓﹕：";

    static String string(int cp) {
        return new String(Character.toChars(cp));
    }

    static int flags(int cp) {
        int flags = 0;
        if (Character.isLetter(cp)) flags |= LETTER;
        if (Character.isDigit(cp)) flags |= DIGIT;
        if (Character.isLetterOrDigit(cp)) flags |= LETTER_OR_DIGIT;
        if (Character.isUpperCase(cp)) flags |= UPPER;
        if (Character.isLowerCase(cp)) flags |= LOWER;
        // Kotlin's Char.isWhitespace().
        if (Character.isWhitespace(cp) || Character.isSpaceChar(cp)) flags |= SPACE;
        if (REGEX_S.matcher(string(cp)).matches()) flags |= REGEX_SPACE;
        if (Character.isTitleCase(cp)) flags |= TITLE;
        if (isCased(cp)) flags |= CASED;
        if (isCaseIgnorable(cp)) flags |= CASE_IGNORABLE;
        return flags;
    }

    static boolean isSurrogate(int cp) {
        return cp >= Character.MIN_SURROGATE && cp <= Character.MAX_SURROGATE;
    }

    static String hex(int value) {
        return String.format("0x%04X", value);
    }

    static String units(String text) {
        var units = new ArrayList<String>();
        for (char c : text.toCharArray()) units.add(hex(c));
        return "&[" + String.join(", ", units) + "]";
    }

    static void field(String doc, String name, List<String> values) {
        System.out.println("    // " + doc);
        System.out.printf("    %s: &[%s],%n", name, String.join(", ", values));
    }

    /** The code points [map] changes and what it changes them to. */
    static void mapping(String what, String name, IntUnaryOperator map) {
        var from = new ArrayList<String>();
        var to = new ArrayList<String>();
        for (int cp = 0; cp <= Character.MAX_CODE_POINT; cp++) {
            int mapped = map.applyAsInt(cp);
            if (mapped != cp) {
                from.add(hex(cp));
                to.add(hex(mapped));
            }
        }
        field("The code points whose " + what + " differs from themselves, ascending.", name + "_from", from);
        field("Their " + what + ", in the same order.", name + "_to", to);
    }

    /** The code points whose [full] mapping, as a string on its own, isn't their [simple] one. */
    static void fullMapping(String what, String name, UnaryOperator<String> full, IntUnaryOperator simple) {
        var from = new ArrayList<String>();
        var to = new ArrayList<String>();
        for (int cp = 0; cp <= Character.MAX_CODE_POINT; cp++) {
            if (isSurrogate(cp)) continue;
            String mapped = full.apply(string(cp));
            if (!mapped.equals(string(simple.applyAsInt(cp)))) {
                from.add(hex(cp));
                to.add(units(mapped));
            }
        }
        field("The code points whose " + what + " on its own isn't their simple one, ascending.", name + "_from", from);
        field("That " + what + "'s UTF-16 units, in the same order.", name + "_to", to);
    }

    /**
     * Where each run of decimal digits starts: `Character.digit(cp, 10)` is 0 there and goes up
     * by one to 9, and is -1 for every code point outside such a run, which this checks.
     */
    static void digits() {
        var zeros = new ArrayList<String>();
        for (int cp = 0; cp <= Character.MAX_CODE_POINT; cp++) {
            int digit = Character.digit(cp, 10);
            if (digit < 0) continue;
            check(digit == 0, "starts a run of digits at " + digit, cp);
            for (int value = 0; value <= 9; value++) {
                check(Character.digit(cp + value, 10) == value, "breaks its run of digits", cp + value);
            }
            zeros.add(hex(cp));
            cp += 9;
        }
        field("The code points where a run of decimal digits 0 to 9 (`Character.digit(cp, 10)`) starts, ascending.", "digit_zeros", zeros);
    }

    /**
     * The first code point of each BMP character's canonical decomposition (Normalizer NFD), where
     * that isn't the character itself. Hangul syllables are left out: the Rust computes theirs,
     * and this checks that it gets the platform's answer.
     */
    static void decompositions() {
        var from = new ArrayList<String>();
        var to = new ArrayList<String>();
        for (int unit = 0; unit <= 0xFFFF; unit++) {
            if (isSurrogate(unit)) continue;
            int first = Normalizer.normalize(string(unit), Normalizer.Form.NFD).codePointAt(0);
            if (unit >= HANGUL_FIRST && unit <= HANGUL_LAST) {
                check(first == 0x1100 + (unit - HANGUL_FIRST) / 588, "Hangul decomposes oddly", unit);
            } else if (first != unit) {
                from.add(hex(unit));
                to.add(hex(first));
            }
        }
        field("The BMP units whose canonical decomposition (NFD) starts with another code point, except Hangul syllables, ascending.", "decomposed_from", from);
        field("That code point, in the same order.", "decomposed_to", to);
    }

    static void check(boolean ok, String what, int unit) {
        if (!ok) throw new IllegalStateException(hex(unit) + " " + what);
    }

    /**
     * Android maps a string with a high surrogate (or, for lower case, İ) through ICU, and any other
     * one char by char. The Rust assumes the two map each BMP character alike, Σ aside, which this
     * checks by putting U+10000 in front of it to send it through ICU.
     */
    static void checkAndroidMapsCharsAsIcuDoes() {
        if (!isAndroid()) return;
        for (int unit = 0; unit <= 0xFFFF; unit++) {
            if (isSurrogate(unit)) continue;
            String alone = string(unit);
            String upper = (UNCASED_SUPPLEMENTARY + alone).toUpperCase(Locale.US).substring(2);
            check(upper.equals(alone.toUpperCase(Locale.US)), "is upper-cased differently by ICU", unit);
            String lower = (UNCASED_SUPPLEMENTARY + alone).toLowerCase(Locale.US).substring(2);
            check(lower.equals(alone.toLowerCase(Locale.US)), "is lower-cased differently by ICU", unit);
        }
    }

    /** Android's API level and ICU and Unicode versions, or the JVM's version. */
    static String platform() {
        try {
            int api = Class.forName("android.os.Build$VERSION").getField("SDK_INT").getInt(null);
            Object icu = Class.forName("android.icu.util.VersionInfo").getField("ICU_VERSION").get(null);
            Object unicode = Class.forName("android.icu.lang.UCharacter").getMethod("getUnicodeVersion").invoke(null);
            return "Android API " + api + " (ICU " + icu + ", Unicode " + unicode + ")";
        } catch (ReflectiveOperationException notAndroid) {
            return "Java " + System.getProperty("java.version");
        }
    }

    public static void main(String[] args) {
        checkAndroidMapsCharsAsIcuDoes();
        var starts = new ArrayList<String>();
        var values = new ArrayList<String>();
        int previous = -1;
        for (int cp = 0; cp <= Character.MAX_CODE_POINT; cp++) {
            int flags = flags(cp);
            if (flags != previous) {
                starts.add(hex(cp));
                values.add(Integer.toString(flags));
                previous = flags;
            }
        }
        System.out.println("//! Generated by tools/JavaChars.java on " + platform() + ". Don't edit;");
        System.out.println("//! run tools/generate-char-tables.sh.");
        System.out.println();
        System.out.println("use super::Tables;");
        System.out.println();
        System.out.println("pub(crate) static TABLES: Tables = Tables {");
        field("Where each run of code points with the same flags starts, ascending from 0.", "flag_starts", starts);
        field("Each run's flags.", "flags", values);
        mapping("upper case (`Character.toUpperCase`)", "upper", Character::toUpperCase);
        mapping("lower case (`Character.toLowerCase`)", "lower", Character::toLowerCase);
        fullMapping("`String.toUpperCase`", "full_upper", s -> s.toUpperCase(Locale.US), Character::toUpperCase);
        fullMapping("`String.toLowerCase`", "full_lower", s -> s.toLowerCase(Locale.US), Character::toLowerCase);
        digits();
        decompositions();
        System.out.println("};");
    }
}
