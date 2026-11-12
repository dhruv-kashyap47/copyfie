# Copyfie — Attribution

The authoritative, generated list of dependencies and the adapted-code table is
`THIRD_PARTY.md` at the repository root. It ships with every build (About →
Open-source notices) and with the installers.

## Adapted prior art

| Upstream | Pinned revision | License | Use |
|---|---|---|---|
| Ortu | `v2.1.0` @ `b5747d3195fd2c52faf296d7489857499fc3021f` | MIT | macOS concealed pasteboard marker list; ideas on focus/paste |
| Ropy | `0.6.2` @ `0d7ac38fcc6883205d400d6849d8977e4f1572e4` | MIT | Ideas on clipboard watching, capture acknowledgement, search |

No upstream code has been copied verbatim. The only row in the adapted-files
table is `platform/macos/src/concealed.rs`, where the marker names are factual
data; the implementation is original.

## Dependency policy

- Allowed: MIT, Apache-2.0, BSD-2/3-Clause, ISC, Zlib, CC0-1.0, Unlicense,
  Unicode-3.0, BSL-1.0.
- MPL-2.0 is allowed for unmodified dependencies only.
- Denied: GPL, AGPL, SSPL, BUSL, Commons Clause, non-commercial, and
  source-available-only licenses.
- Enforced by `cargo-deny` in CI against the full transitive graph, and
  `npm audit --omit=dev` for runtime JavaScript.

Linux system libraries (WebKitGTK, GTK) are dynamically linked distro packages
and are not bundled; list them for counsel review before v1.0.

## Obligations

- Keep `NOTICE`, `THIRD_PARTY.md`, and `third_party/licenses/*.txt` in release
  assets and installers.
- Render the notices in About.
- MIT headers on any file that copies upstream code, plus a provenance row.
- Never add GPL/AGPL headers or dependencies.

## Trademarks

Copyfie does not use the names or logos of Ortu or Ropy. They are credited as
prior art only.
