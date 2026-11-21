<img align="left" width="80" height="80"
src=".github/repo_icon.png" alt="App icon">

# AudioWhisper Keyboard [![CI](https://github.com/jtn0123/AudioWhisper-Keyboard/actions/workflows/android.yml/badge.svg?branch=main)](https://github.com/jtn0123/AudioWhisper-Keyboard/actions/workflows/android.yml)

A personal, sideloaded fork of [FlorisBoard](https://github.com/florisboard/florisboard) v0.5.2,
the Android companion to the AudioWhisper Mac app. It installs next to stock FlorisBoard as
`com.jtn0123.audiowhisper.keyboard`. What it adds:

- **Dictation**, on the phone: NVIDIA's Parakeet TDT 0.6B v2 through sherpa-onnx. Filler words
  are removed and a custom vocabulary applied, by the Mac app's rules.
- **Read aloud**: a speaker beside the mic reads the screen, in a built-in neural voice (Kyutai's
  Pocket TTS) or the phone's own.
- **Latin-script input**, including accented layouts and Vietnamese Telex; numeric, symbol and phone layouts, emoji, add-ons and the theme editor remain available
- **Autocorrect and predictions** from AOSP LatinIME, with the typing rules rewritten in Rust
  (`lib/rust`, see [docs/rust.md](docs/rust.md)).
- **GIF search** through GIPHY.

The target is 90%+ Rust, including keyboard drawing and settings models. Today Compose still
draws the keyboard; the planned Rust renderer must pass behaviour, accessibility and phone
performance checks before replacing it. Kotlin will retain the Android service/API bridges,
draw surface, touch forwarding and accessible settings controls. See [the staged plan](docs/rust.md)
and [the decision](docs/decisions.md).

Our code is under `app/src/main/kotlin/com/jtn0123/audiowhisper` and `lib/rust`. Changes to
upstream files are marked `AudioWhisper:`.

## Building

You need:

- **JDK 24.** The Rust character tables are checked against the JVM's, which follow its Unicode
  version.
- **The Android SDK with NDK 28.1.13356709**, pinned in `gradle/tools.versions.toml`.
- **[rustup](https://rustup.rs).** The first build installs the Rust release and Android target
  pinned in `lib/rust/rust-toolchain.toml`.
- **Git LFS**, for `app/libs/sherpa-onnx-*.aar`.

Nothing else comes from your machine. jetpref, a snapshot Sonatype no longer serves, is kept in
`third_party/m2`.

```bash
./gradlew assembleDebug                               # the debug build, used day to day
./gradlew assemblePersonal                            # shrunk and optimised, not debuggable
./gradlew --stacktrace test :app:lintDebug :lib:rust:check :lib:rust:oracleCheck :app:checkNativeDeclarations :app:checkJvmLedger assembleDebug :app:compileDebugAndroidTestKotlin
```

**GIPHY** needs a key, as `giphy.apiKey=...` in `local.properties` (never committed) or the
`GIPHY_API_KEY` environment variable. Without one the build works and GIF search finds nothing.

## Installing on a phone

```bash
scripts/install-dev.sh [--personal] [adb-serial]
```

It installs as an update, so settings and the models survive. It backs settings up to
`.device-backup/` first and makes this the default keyboard. See the script's header for the
details.

The models aren't in the APK. They are copied into the app's private files over adb with
`run-as`, which needs the **debug** build on the phone. Install the personal build over it
afterwards; it keeps them.

- **Speech model** (about 660 MB). Get `sherpa-onnx-nemo-parakeet-tdt-0.6b-v2-int8` from
  sherpa-onnx's releases (asr-models) and unpack it. Then copy `encoder.int8.onnx`,
  `decoder.int8.onnx`, `joiner.int8.onnx` and `tokens.txt` to
  `files/models/sherpa-onnx-nemo-parakeet-tdt-0.6b-v2-int8/`:
  ```bash
  adb push <folder>/<file> /data/local/tmp/
  adb shell run-as com.jtn0123.audiowhisper.keyboard sh -c \
    "mkdir -p files/models/sherpa-onnx-nemo-parakeet-tdt-0.6b-v2-int8 && cp /data/local/tmp/<file> files/models/sherpa-onnx-nemo-parakeet-tdt-0.6b-v2-int8/"
  ```
- **Neural voice** (about 190 MB): `scripts/install-voice.sh <unpacked-pocket-tts-folder>`.

Settings › Dictation shows whether each model is installed and whole. Each file is checked
against the size and SHA-256 in `ModelFiles.kt` and `VoiceEngine.kt`.

## Testing on an emulator

The `aw_keyboard` AVD is an arm64 Pixel image, so the phone's native libraries run as they are.
[tools/voice-bench/README.md](tools/voice-bench/README.md) describes `s24_like`, which matches the
S24 Ultra's cores, memory and screen, and how to time the voice with `scripts/bench-voice.sh`.

## More

- [docs/decisions.md](docs/decisions.md): why Android 16, GIPHY, arm64 only, Rust, the
  neural voice's settings, and the debug and personal builds, and what would change each.
- [docs/rust.md](docs/rust.md): the Rust rewrite, its checks, and how it is compared with the
  code it replaced.
- [third_party/m2/README.md](third_party/m2/README.md): the kept copy of jetpref, and how to
  update it.

---

*The upstream FlorisBoard README follows.*

<h2>FlorisBoard</h2>

**FlorisBoard** is a free and open-source keyboard for Android 8.0+
devices. It aims at being modern, user-friendly and customizable while
fully respecting your privacy. Currently in beta state.

<table>
<tr>
<th align="center" width="50%">
<h3>Stable <a href="https://github.com/florisboard/florisboard/releases/latest"><img alt="Latest stable release" src="https://img.shields.io/github/v/release/florisboard/florisboard?sort=semver&display_name=tag&color=28a745"></a></h3>
</th>
<th align="center" width="50%">
<h3>Preview <a href="https://github.com/florisboard/florisboard/releases"><img alt="Latest preview release" src="https://img.shields.io/github/v/release/florisboard/florisboard?include_prereleases&sort=semver&display_name=tag&color=fd7e14"></a></h3>
</th>
</tr>
<tr>
<td valign="top">
<p><i>Major versions only</i><br><br>Updates are more polished, new features are matured and tested through to ensure a stable experience.</p>
</td>
<td valign="top">
<p><i>Major + Alpha/Beta/Rc versions</i><br><br>Updates contain new features that may not be fully matured yet and bugs are more likely to occur. Allows you to give early feedback.</p>
</td>
</tr>
<tr>
<td valign="top">
<p>
<a href="https://apt.izzysoft.de/fdroid/index/apk/dev.patrickgold.florisboard"><img src="https://gitlab.com/IzzyOnDroid/repo/-/raw/master/assets/IzzyOnDroid.png" height="64" alt="IzzySoft repo badge"></a>
<a href="https://f-droid.org/packages/dev.patrickgold.florisboard"><img src="https://fdroid.gitlab.io/artwork/badge/get-it-on.png" height="64" alt="F-Droid badge"></a>
</p>
<p>

**Google Play**: Join the [FlorisBoard Test Group](https://groups.google.com/g/florisboard-closed-beta-test), then visit the [testing page](https://play.google.com/apps/testing/dev.patrickgold.florisboard). Once joined and installed, updates will be delivered like for any other app. ([Store entry](https://play.google.com/store/apps/details?id=dev.patrickgold.florisboard))

</p>
<p>

**Obtainium**: [Auto-import stable config][obtainium_stable]

</p>
<p>

**Manual**: Download and install the APK from the release page.

</p>
</td>
<td valign="top">
<p><a href="https://apt.izzysoft.de/fdroid/index/apk/dev.patrickgold.florisboard.beta"><img src="https://gitlab.com/IzzyOnDroid/repo/-/raw/master/assets/IzzyOnDroid.png" height="64" alt="IzzySoft repo badge"></a></p>
<p>

**Google Play**: Join the [FlorisBoard Test Group](https://groups.google.com/g/florisboard-closed-beta-test), then visit the [preview testing page](https://play.google.com/apps/testing/dev.patrickgold.florisboard.beta). Once joined and installed, updates will be delivered like for any other app. ([Store entry](https://play.google.com/store/apps/details?id=dev.patrickgold.florisboard.beta))

</p>
<p>

**Obtainium**: [Auto-import preview config][obtainium_preview]

</p>
<p>

**Manual**: Download and install the APK from the release page.

</p>
</td>
</tr>
</table>

Beginning with v0.7 FlorisBoard will enter the public beta on Google Play.

## Highlighted features
- Integrated clipboard manager / history
- Advanced theming support and customization
- Integrated extension support (still evolving)
- Emoji keyboard / history / suggestions

> [!IMPORTANT]
> Word suggestions/spell checking are not included in the current releases
> and are a major goal for the v0.6 milestone.

Feature roadmap: See [ROADMAP.md](ROADMAP.md)

## Contributing
Want to contribute to FlorisBoard? That's great to hear! There are lots of
different ways to help out, please see the [contribution guidelines](CONTRIBUTING.md) for more info.

## Addons Store
The official [Addons Store](https://beta.addons.florisboard.org) offers the possibility for the community to share and download FlorisBoard extensions.
Instructions on how to publish addons can be found [here](https://github.com/florisboard/florisboard/wiki/How-to-publish-on-FlorisBoard-Addons).

Many thanks to Ali ([@4H1R](https://github.com/4H1R)) for implementing the store!

> [!NOTE]
> During the initial beta release phase, the Addons Store _will_ only accept theme extensions.
> Later on we plan to add support for language packs and keyboard extensions.

## List of permissions FlorisBoard requests
Please refer to this [page](https://github.com/florisboard/florisboard/wiki/List-of-permissions-FlorisBoard-requests)
to get more information on this topic.

## APK signing certificate hashes

The package names and SHA-256 hashes of the signature certificate are listed below, so you can verify both FlorisBoard variants with apksigner by using `apksigner verify --print-certs florisboard-<version>-<track>.apk` when you download the APK.
If you have [AppVerifier](https://github.com/soupslurpr/AppVerifier) installed, you can alternatively copy both the package name and the hash of the corresponding track and share them to AppVerifier.

##### Stable track:

dev.patrickgold.florisboard<br>
0B:80:71:64:50:8E:AF:EB:1F:BB:81:5B:E7:A2:3C:77:FE:68:9D:94:B1:43:75:C9:9B:DA:A9:B6:57:7F:D6:D6

##### Preview track:

dev.patrickgold.florisboard.beta<br>
0B:80:71:64:50:8E:AF:EB:1F:BB:81:5B:E7:A2:3C:77:FE:68:9D:94:B1:43:75:C9:9B:DA:A9:B6:57:7F:D6:D6


## Used libraries, components and icons
* [AndroidX libraries](https://github.com/androidx/androidx) by
  [Android Jetpack](https://github.com/androidx)
* [AboutLibraries](https://github.com/mikepenz/AboutLibraries) by
  [mikepenz](https://github.com/mikepenz)
* [Google Material icons](https://github.com/google/material-design-icons) by
  [Google](https://github.com/google)
* [JetPref preference library](https://github.com/patrickgold/jetpref) by
  [patrickgold](https://github.com/patrickgold)
* [KotlinX coroutines library](https://github.com/Kotlin/kotlinx.coroutines) by
  [Kotlin](https://github.com/Kotlin)
* [KotlinX serialization library](https://github.com/Kotlin/kotlinx.serialization) by
  [Kotlin](https://github.com/Kotlin)
* [sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx) by [k2-fsa](https://github.com/k2-fsa), for
  dictation and the read-aloud voice
* [Pocket TTS](https://github.com/kyutai-labs/pocket-tts) by [Kyutai](https://github.com/kyutai-labs),
  the read-aloud voice (copied onto the phone, not shipped in the app; CC-BY 4.0, ONNX export by
  [KevinAHM](https://huggingface.co/KevinAHM/pocket-tts-onnx)), speaking in the voice of the "Loona"
  sample from [delayed-streams-modeling](https://github.com/kyutai-labs/delayed-streams-modeling)

Many thanks to [Nikolay Anzarov](https://www.behance.net/nikolayanzarov) ([@BloodRaven0](https://github.com/BloodRaven0)) for designing and providing the main app icons to this project!

## License
```
Copyright 2020-2025 The FlorisBoard Contributors

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```

Thanks to [The FlorisBoard Contributors](https://github.com/florisboard/florisboard/graphs/contributors) for making this project possible!

<!-- BEGIN SECTION: obtainium_links -->
<!-- auto-generated link templates, do NOT edit by hand -->
<!-- see fastlane/update-readme.sh -->
[obtainium_preview]: https://apps.obtainium.imranr.dev/redirect.html?r=obtainium://app/%7B%22id%22%3A%22dev.patrickgold.florisboard.beta%22%2C%22url%22%3A%22https%3A%2F%2Fgithub.com%2Fflorisboard%2Fflorisboard%22%2C%22author%22%3A%22florisboard%22%2C%22name%22%3A%22FlorisBoard%20Preview%22%2C%22additionalSettings%22%3A%22%7B%5C%22includePrereleases%5C%22%3Atrue%2C%5C%22fallbackToOlderReleases%5C%22%3Atrue%2C%5C%22apkFilterRegEx%5C%22%3A%5C%22preview%5C%22%7D%22%7D%0A
[obtainium_stable]: https://apps.obtainium.imranr.dev/redirect.html?r=obtainium://app/%7B%22id%22%3A%22dev.patrickgold.florisboard%22%2C%22url%22%3A%22https%3A%2F%2Fgithub.com%2Fflorisboard%2Fflorisboard%22%2C%22author%22%3A%22florisboard%22%2C%22name%22%3A%22FlorisBoard%20Stable%22%2C%22additionalSettings%22%3A%22%7B%5C%22includePrereleases%5C%22%3Afalse%2C%5C%22fallbackToOlderReleases%5C%22%3Atrue%2C%5C%22apkFilterRegEx%5C%22%3A%5C%22stable%5C%22%7D%22%7D%0A
<!-- END SECTION: obtainium_links -->
