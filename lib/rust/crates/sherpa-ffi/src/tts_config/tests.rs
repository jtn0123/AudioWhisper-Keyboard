//! ABI values independently emitted by the C compiler for the pinned header.
use super::*;
use std::mem::{align_of, offset_of, size_of};
#[test]
fn exact_v2_0_1_c_layout() {
    assert_eq!(size_of::<OfflineTtsVitsModelConfig>(), 56);
    assert_eq!(align_of::<OfflineTtsVitsModelConfig>(), 8);
    assert_eq!(offset_of!(OfflineTtsVitsModelConfig, model), 0);
    assert_eq!(offset_of!(OfflineTtsVitsModelConfig, lexicon), 8);
    assert_eq!(offset_of!(OfflineTtsVitsModelConfig, tokens), 16);
    assert_eq!(offset_of!(OfflineTtsVitsModelConfig, data_dir), 24);
    assert_eq!(offset_of!(OfflineTtsVitsModelConfig, noise_scale), 32);
    assert_eq!(offset_of!(OfflineTtsVitsModelConfig, noise_scale_w), 36);
    assert_eq!(offset_of!(OfflineTtsVitsModelConfig, length_scale), 40);
    assert_eq!(offset_of!(OfflineTtsVitsModelConfig, dict_dir), 48);
    assert_eq!(size_of::<OfflineTtsMatchaModelConfig>(), 56);
    assert_eq!(align_of::<OfflineTtsMatchaModelConfig>(), 8);
    assert_eq!(offset_of!(OfflineTtsMatchaModelConfig, acoustic_model), 0);
    assert_eq!(offset_of!(OfflineTtsMatchaModelConfig, vocoder), 8);
    assert_eq!(offset_of!(OfflineTtsMatchaModelConfig, lexicon), 16);
    assert_eq!(offset_of!(OfflineTtsMatchaModelConfig, tokens), 24);
    assert_eq!(offset_of!(OfflineTtsMatchaModelConfig, data_dir), 32);
    assert_eq!(offset_of!(OfflineTtsMatchaModelConfig, noise_scale), 40);
    assert_eq!(offset_of!(OfflineTtsMatchaModelConfig, length_scale), 44);
    assert_eq!(offset_of!(OfflineTtsMatchaModelConfig, dict_dir), 48);
    assert_eq!(size_of::<OfflineTtsKokoroModelConfig>(), 64);
    assert_eq!(align_of::<OfflineTtsKokoroModelConfig>(), 8);
    assert_eq!(offset_of!(OfflineTtsKokoroModelConfig, model), 0);
    assert_eq!(offset_of!(OfflineTtsKokoroModelConfig, voices), 8);
    assert_eq!(offset_of!(OfflineTtsKokoroModelConfig, tokens), 16);
    assert_eq!(offset_of!(OfflineTtsKokoroModelConfig, data_dir), 24);
    assert_eq!(offset_of!(OfflineTtsKokoroModelConfig, length_scale), 32);
    assert_eq!(offset_of!(OfflineTtsKokoroModelConfig, dict_dir), 40);
    assert_eq!(offset_of!(OfflineTtsKokoroModelConfig, lexicon), 48);
    assert_eq!(offset_of!(OfflineTtsKokoroModelConfig, lang), 56);
    assert_eq!(size_of::<OfflineTtsKittenModelConfig>(), 48);
    assert_eq!(align_of::<OfflineTtsKittenModelConfig>(), 8);
    assert_eq!(offset_of!(OfflineTtsKittenModelConfig, model), 0);
    assert_eq!(offset_of!(OfflineTtsKittenModelConfig, voices), 8);
    assert_eq!(offset_of!(OfflineTtsKittenModelConfig, tokens), 16);
    assert_eq!(offset_of!(OfflineTtsKittenModelConfig, data_dir), 24);
    assert_eq!(offset_of!(OfflineTtsKittenModelConfig, length_scale), 32);
    assert_eq!(offset_of!(OfflineTtsKittenModelConfig, lexicon), 40);
    assert_eq!(size_of::<OfflineTtsZipvoiceModelConfig>(), 64);
    assert_eq!(align_of::<OfflineTtsZipvoiceModelConfig>(), 8);
    assert_eq!(offset_of!(OfflineTtsZipvoiceModelConfig, tokens), 0);
    assert_eq!(offset_of!(OfflineTtsZipvoiceModelConfig, encoder), 8);
    assert_eq!(offset_of!(OfflineTtsZipvoiceModelConfig, decoder), 16);
    assert_eq!(offset_of!(OfflineTtsZipvoiceModelConfig, vocoder), 24);
    assert_eq!(offset_of!(OfflineTtsZipvoiceModelConfig, data_dir), 32);
    assert_eq!(offset_of!(OfflineTtsZipvoiceModelConfig, lexicon), 40);
    assert_eq!(offset_of!(OfflineTtsZipvoiceModelConfig, feat_scale), 48);
    assert_eq!(offset_of!(OfflineTtsZipvoiceModelConfig, t_shift), 52);
    assert_eq!(offset_of!(OfflineTtsZipvoiceModelConfig, target_rms), 56);
    assert_eq!(
        offset_of!(OfflineTtsZipvoiceModelConfig, guidance_scale),
        60
    );
    assert_eq!(size_of::<OfflineTtsPocketModelConfig>(), 64);
    assert_eq!(align_of::<OfflineTtsPocketModelConfig>(), 8);
    assert_eq!(offset_of!(OfflineTtsPocketModelConfig, lm_flow), 0);
    assert_eq!(offset_of!(OfflineTtsPocketModelConfig, lm_main), 8);
    assert_eq!(offset_of!(OfflineTtsPocketModelConfig, encoder), 16);
    assert_eq!(offset_of!(OfflineTtsPocketModelConfig, decoder), 24);
    assert_eq!(
        offset_of!(OfflineTtsPocketModelConfig, text_conditioner),
        32
    );
    assert_eq!(offset_of!(OfflineTtsPocketModelConfig, vocab_json), 40);
    assert_eq!(
        offset_of!(OfflineTtsPocketModelConfig, token_scores_json),
        48
    );
    assert_eq!(
        offset_of!(OfflineTtsPocketModelConfig, voice_embedding_cache_capacity),
        56
    );
    assert_eq!(size_of::<OfflineTtsSupertonicModelConfig>(), 56);
    assert_eq!(align_of::<OfflineTtsSupertonicModelConfig>(), 8);
    assert_eq!(
        offset_of!(OfflineTtsSupertonicModelConfig, duration_predictor),
        0
    );
    assert_eq!(offset_of!(OfflineTtsSupertonicModelConfig, text_encoder), 8);
    assert_eq!(
        offset_of!(OfflineTtsSupertonicModelConfig, vector_estimator),
        16
    );
    assert_eq!(offset_of!(OfflineTtsSupertonicModelConfig, vocoder), 24);
    assert_eq!(offset_of!(OfflineTtsSupertonicModelConfig, tts_json), 32);
    assert_eq!(
        offset_of!(OfflineTtsSupertonicModelConfig, unicode_indexer),
        40
    );
    assert_eq!(offset_of!(OfflineTtsSupertonicModelConfig, voice_style), 48);
    assert_eq!(size_of::<OfflineTtsModelConfig>(), 424);
    assert_eq!(align_of::<OfflineTtsModelConfig>(), 8);
    assert_eq!(offset_of!(OfflineTtsModelConfig, vits), 0);
    assert_eq!(offset_of!(OfflineTtsModelConfig, num_threads), 56);
    assert_eq!(offset_of!(OfflineTtsModelConfig, debug), 60);
    assert_eq!(offset_of!(OfflineTtsModelConfig, provider), 64);
    assert_eq!(offset_of!(OfflineTtsModelConfig, matcha), 72);
    assert_eq!(offset_of!(OfflineTtsModelConfig, kokoro), 128);
    assert_eq!(offset_of!(OfflineTtsModelConfig, kitten), 192);
    assert_eq!(offset_of!(OfflineTtsModelConfig, zipvoice), 240);
    assert_eq!(offset_of!(OfflineTtsModelConfig, pocket), 304);
    assert_eq!(offset_of!(OfflineTtsModelConfig, supertonic), 368);
    assert_eq!(size_of::<OfflineTtsConfig>(), 456);
    assert_eq!(align_of::<OfflineTtsConfig>(), 8);
    assert_eq!(offset_of!(OfflineTtsConfig, model), 0);
    assert_eq!(offset_of!(OfflineTtsConfig, rule_fsts), 424);
    assert_eq!(offset_of!(OfflineTtsConfig, max_num_sentences), 432);
    assert_eq!(offset_of!(OfflineTtsConfig, rule_fars), 440);
    assert_eq!(offset_of!(OfflineTtsConfig, silence_scale), 448);
    assert_eq!(size_of::<GenerationConfig>(), 104);
    assert_eq!(align_of::<GenerationConfig>(), 8);
    assert_eq!(offset_of!(GenerationConfig, silence_scale), 0);
    assert_eq!(offset_of!(GenerationConfig, speed), 4);
    assert_eq!(offset_of!(GenerationConfig, sid), 8);
    assert_eq!(offset_of!(GenerationConfig, reference_audio), 16);
    assert_eq!(offset_of!(GenerationConfig, reference_audio_len), 24);
    assert_eq!(offset_of!(GenerationConfig, reference_sample_rate), 28);
    assert_eq!(offset_of!(GenerationConfig, reference_text), 32);
    assert_eq!(offset_of!(GenerationConfig, num_steps), 40);
    assert_eq!(offset_of!(GenerationConfig, extra), 48);
    assert_eq!(offset_of!(GenerationConfig, phoneme_codepoints), 56);
    assert_eq!(offset_of!(GenerationConfig, phoneme_codepoints_lens), 64);
    assert_eq!(
        offset_of!(GenerationConfig, phoneme_codepoints_num_sentences),
        72
    );
    assert_eq!(offset_of!(GenerationConfig, tokens), 80);
    assert_eq!(offset_of!(GenerationConfig, tokens_lens), 88);
    assert_eq!(offset_of!(GenerationConfig, tokens_num_sentences), 96);
    assert_eq!(size_of::<GeneratedAudio>(), 16);
    assert_eq!(align_of::<GeneratedAudio>(), 8);
    assert_eq!(offset_of!(GeneratedAudio, samples), 0);
    assert_eq!(offset_of!(GeneratedAudio, n), 8);
    assert_eq!(offset_of!(GeneratedAudio, sample_rate), 12);
}
