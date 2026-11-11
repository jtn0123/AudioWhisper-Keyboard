# jetpref, kept in the repo

FlorisBoard v0.5.2 depends on the jetpref preference library at
`20251119T222500Z-SNAPSHOT` (`gradle/libs.versions.toml`). Upstream took it from
Sonatype's snapshot repository, which deletes old snapshots, so it can no longer
be downloaded. This directory is a Maven repository holding that one version,
and `settings.gradle.kts` resolves the `dev.patrickgold.jetpref` group from here
and from nowhere else.

**Where it came from:** published to `~/.m2` on 2026-11-06 from jetpref's
source (github.com/patrickgold/jetpref) at the snapshot named above. The
`.module` files record each artifact's SHA-256; Gradle checks the jars and
AARs against them.

Sources and javadoc jars are left out. Only the jars, AARs, POMs and Gradle
module files are kept, about 700 KB.

**To update:** build and publish the new jetpref to `~/.m2` with
`./gradlew publishToMavenLocal` in a jetpref checkout, then:
1. Copy `~/.m2/repository/dev/patrickgold/jetpref` here, without `*-sources.jar`,
   `*-javadoc.jar` and `maven-metadata-local.xml`.
2. Bump `patrickgold-jetpref` in `gradle/libs.versions.toml`.
