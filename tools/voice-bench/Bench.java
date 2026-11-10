import com.k2fsa.sherpa.onnx.*;
import java.nio.file.*;
import java.util.*;
import kotlin.jvm.functions.Function1;

/**
 * Times Pocket TTS the way read-aloud uses it: one generateWithConfigAndCallback per sentence.
 * Usage: Bench <modelDir> <reference.wav> <threads> <numSteps> <textFile> [key=value extra...]
 * Prints, per sentence: chars, audio s, generation s, first-audio s; then totals.
 */
public class Bench {
    static final class Cb implements Function1<float[], Integer> {
        long first = -1; long samples = 0; final long start;
        Cb(long start) { this.start = start; }
        public Integer invoke(float[] s) {
            if (first < 0) first = System.nanoTime() - start;
            samples += s.length;
            return 1;
        }
    }

    public static void main(String[] a) throws Exception {
        String dir = a[0];
        OfflineTtsPocketModelConfig p = new OfflineTtsPocketModelConfig();
        p.setLmFlow(dir + "/lm_flow.int8.onnx");
        p.setLmMain(dir + "/lm_main.int8.onnx");
        p.setEncoder(dir + "/encoder.onnx");
        p.setDecoder(dir + "/decoder.int8.onnx");
        p.setTextConditioner(dir + "/text_conditioner.onnx");
        p.setVocabJson(dir + "/vocab.json");
        p.setTokenScoresJson(dir + "/token_scores.json");
        OfflineTtsModelConfig m = new OfflineTtsModelConfig();
        m.setPocket(p);
        m.setNumThreads(Integer.parseInt(a[2]));
        OfflineTtsConfig c = new OfflineTtsConfig();
        c.setModel(m);
        long t0 = System.nanoTime();
        OfflineTts tts = new OfflineTts(null, c);
        System.out.printf("load %.2f s%n", (System.nanoTime() - t0) / 1e9);
        WaveData ref = WaveReader.Companion.readWave(a[1]);
        GenerationConfig g = new GenerationConfig();
        g.setReferenceAudio(ref.getSamples());
        g.setReferenceSampleRate(ref.getSampleRate());
        g.setNumSteps(Integer.parseInt(a[3]));
        Map<String, String> extra = new HashMap<>();
        for (int i = 5; i < a.length; i++) {
            String[] kv = a[i].split("=", 2);
            extra.put(kv[0], kv[1]);
        }
        if (!extra.isEmpty()) g.setExtra(extra);
        List<String> lines = Files.readAllLines(Paths.get(a[4]));
        int rate = tts.sampleRate();
        double audio = 0, gen = 0, worstGap = 0, firstLatency = -1;
        // Playback model: audio queues back to back; a gap is when a sentence is ready after the
        // previous one has finished playing.
        double playEnd = -1, wall = 0;
        for (String line : lines) {
            if (line.isBlank()) continue;
            long s = System.nanoTime();
            Cb cb = new Cb(s);
            tts.generateWithConfigAndCallback(line, g, cb);
            double took = (System.nanoTime() - s) / 1e9;
            double secs = cb.samples / (double) rate;
            double firstAt = wall + cb.first / 1e9;
            if (playEnd < 0) { firstLatency = firstAt; playEnd = firstAt; }
            double gap = Math.max(0, firstAt - playEnd);
            worstGap = Math.max(worstGap, gap);
            playEnd = Math.max(playEnd, firstAt) + secs;
            wall += took;
            audio += secs; gen += took;
            System.out.printf("chars %3d audio %5.2f gen %5.2f first %5.2f gap %4.2f%n",
                    line.length(), secs, took, cb.first / 1e9, gap);
        }
        System.out.printf("TOTAL audio %.1f s gen %.1f s speed x%.2f startup %.2f s worstGap %.2f s%n",
                audio, gen, audio / gen, firstLatency, worstGap);
        tts.release();
    }
}
