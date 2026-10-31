# Uploaded Ghidra analysis reference

The user explicitly asked to use the corrected Ghidra export going forward.
Use it alongside the ROM, instruction listings and routine-level oracles.

| Field | Verified value |
| --- | --- |
| Upload | `sediment://file_00000000181881f5a9d1370f120f2f2a` |
| Filename | `ShadowRunBin.gzf` |
| File SHA-256 | `2d40f6cdca5dc7a94a7d4e877e3cfdecaac51dd70127950619b97b84a9e03966` |
| Size | 3,086,341 bytes |
| Ghidra version | 12.2 |
| Language | `68000:BE:32:default`, compiler `default` |
| Program range | `00000000..001fffff` |
| Program ROM SHA-256 | `92a614491fc3ea12a6df32459cce08b48574b7b814e244c07eb860dc5c6edb7e` |
| Existing functions on import | 607 |

The database's recorded hash and metadata were checked initially. The companion
milestone then imported the GZF through Ghidra's GZF loader and independently
hashed all 2 MB of actual program memory, confirming that it matches the ROM.
The user's uploaded file is preserved; analysis runs on a private working copy.
The earlier `ShadowrunBinary.gzf` imported a ZIP as x86 and is not the reference.

Prior runtime paths (check existence; do not assume they survive):

- Upload: `/workspace/attachments/2dd1274e-3024-444c-84a2-72f8294dc9dd/ShadowRunBin.gzf`
- Tool: `/workspace/reverse-tools/ghidra_12.2_PUBLIC`
- Java: `/workspace/reverse-tools/jdk-25.0.5+8`
- Private project: `artifacts/ghidra-projects/ShadowrealmReference.gpr`
- Exported reference views: `artifacts/ghidra-companion`

If the upload needs downloading again, use the Sediment file ID above. Do not
commit this GZF: its database includes game bytes. Generated function listings,
decompiled C and project databases stay under ignored `artifacts/`.

## Toolchain provenance

Official Ghidra archive:
`https://github.com/NationalSecurityAgency/ghidra/releases/download/Ghidra_12.2_build/ghidra_12.2_PUBLIC_20261016.zip`

SHA-256, verified against the official release assets page:
`2c2cd776db3c1f6a097ba418922378ac3649cadd5e844a6f5f088ae628a7be67`.

Ghidra 12.2 requires Java 25. The system Java 21 is insufficient. A workspace-only
Temurin JDK was downloaded from the official `adoptium/temurin25-binaries` release
`jdk-25.0.5+8`, file `OpenJDK25U-jdk_x64_linux_hotspot_25.0.5_8.tar.gz`.
Its published SHA-256 was verified:
`68982529869257d0ee6b824ec5aa270ff8b118bb438670da0b97ebdab178d7fb`.
Neither tool is required on the user's Windows PC or by the Rust game.

## Reproduce a targeted export

`tools/ghidra/ExportCompanion.java` verifies language/source memory and exports
targeted function views from a private project, creating missing entry definitions
only in that working copy. The output directory must be new.

```sh
mkdir -p artifacts/ghidra-projects artifacts/ghidra-user
JAVA_HOME=/workspace/reverse-tools/jdk-25.0.5+8 \
JAVA_TOOL_OPTIONS="${JAVA_TOOL_OPTIONS:+$JAVA_TOOL_OPTIONS }-Duser.home=/workspace/Shadowrealm/artifacts/ghidra-user" \
PATH=/workspace/reverse-tools/jdk-25.0.5+8/bin:$PATH \
/workspace/reverse-tools/ghidra_12.2_PUBLIC/support/analyzeHeadless \
  artifacts/ghidra-projects ReferenceCopy \
  -import /path/to/ShadowRunBin.gzf -noanalysis \
  -scriptPath tools/ghidra \
  -postScript ExportCompanion.java artifacts/companion-reference
```

For an existing private project, use `-process ShadowRunBin -noanalysis` instead
of `-import`. Optional additional script arguments are hexadecimal function
entries. The default set covers companion dispatch, formation, steering, regular
routes and movement. Restore executable modes when extracting the official ZIP
with a tool that does not preserve them.

Ghidra's recovered calling conventions do not fully describe these routines'
register inputs/outputs. Some decompiled C drops important register arithmetic
or displays unknown condition flags. Use its function boundaries/references and
instruction views, then verify the translation with the ROM oracle. Do not copy
its untyped C into Rust and assume it is equivalent. This milestone does not
execute Ghidra, its decompiler, or emulated game code in the desktop runtime.
