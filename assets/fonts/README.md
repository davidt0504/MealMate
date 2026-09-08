# MVP-034 font and palette provenance

These fonts are bundled assets. Kimatta never fetches fonts at runtime.

## Font source and subsetting

- Source: `google/fonts` commit `5e35378e6bda803962ee6fd257e444a7d459660d`
- Tool: FontTools `4.64.0`, installed in an isolated temporary Python environment
- Source paths: `ofl/<family>/<Family>-{Regular,Bold}.ttf` and `OFL.txt`
- Retained Unicode ranges: `U+0000-024F,U+1E00-1EFF,U+2000-206F,U+20A0-20CF,`
  `U+2100-214F,U+2190-21FF,U+2200-22FF,U+25A0-25FF,U+2600-26FF,U+2700-27BF`
- Retained tables/features: all layout features, glyph names, symbol and legacy cmaps,
  all name IDs/languages, `.notdef` glyph and outline, recommended glyphs, hinting,
  recalculated average width and maximum context.

The exact `pyftsubset` options after the input and output paths were:

```text
--unicodes=<ranges-above> --layout-features=* --glyph-names --symbol-cmap
--legacy-cmap --name-IDs=* --name-legacy --name-languages=* --notdef-glyph
--notdef-outline --recommended-glyphs --recalc-average-width --recalc-max-context
```

| File | Source bytes | Source SHA-256 | Subset bytes | Subset SHA-256 |
|---|---:|---|---:|---|
| AtkinsonHyperlegible-Regular.ttf | 54,348 | `7fb917c89019896d0b52ee84b7cbb3304c18cb90b19a62f5e32712bd23e97669` | 48,464 | `6220409ec06fa97f270d8fc58bc7a2e245d8fcfdbe0a4c58a7aa6468a975268b` |
| AtkinsonHyperlegible-Bold.ttf | 55,256 | `5a3b0c8cc8ca545155150b4512a1fa248298df121c50d6557e651e61fbdab92f` | 49,396 | `881f32ef9e42e18e2b7f6bf0a147e2f833de2044a47a96d8f36cf1bfa44703e0` |
| ShipporiMincho-Regular.ttf | 8,677,284 | `769b5269f0f9bc6534b352c0e6bd856a566e03ff788f107191c2d835863570b2` | 128,612 | `36685c16a0a9797b1174f9aca10d061f703f6a82f59d80274e0d1a57288fc454` |
| ShipporiMincho-Bold.ttf | 8,563,788 | `63bc4eddc74793f671c3ab827c5175e773ffbe569d0bf50ee65375ea9e3bc286` | 128,684 | `10dcbc40b03246f4eb26ae50a42c762451d562d9466e04b390382ebc0ee1a5ff` |
| ZenOldMincho-Regular.ttf | 5,442,512 | `4c051a78a21c4e8e9dccf1c754776d33f356b8cc6ef95d9b64761b9bae814b84` | 61,372 | `a953ac1ff1f0438f47fcf67e0f3d33a077c0262a3d2b9013667cff0c5c042573` |
| ZenOldMincho-Bold.ttf | 5,436,460 | `d6b95c1ff45c8dac153d28961e4c37d7d03b648330c71f884d124dc652a13c0d` | 60,916 | `129bf97e8a73f0e986e96881fb4d27739d00b7dbb5d9b6a40998b69669011c1d` |
| ZenMaruGothic-Regular.ttf | 3,832,756 | `a0c0b53543e0993ae2225e629c833f3d51495ad31720694ff112ce4ce11111ef` | 57,664 | `da764b6c252bdb7934da289cc46b85beb5ba799b17ff01dae27f65cb3c442f84` |
| ZenMaruGothic-Bold.ttf | 3,779,008 | `fe24426b9c8b5523a0146a8235c8674eccf0493af354a53ec895c3596d9eb745` | 57,740 | `27b0cec5bb2de4d1674b7d46a1e381ccf27267648b623d7a81e69e01e49ba6f6` |

The unmodified OFL files are stored beside their corresponding faces, bundled through
`pubspec.yaml`, and registered with `LicenseRegistry` so Flutter's in-app license page exposes them.

## Palette anchors

All other semantic colors are explicit sRGB tokens in `lib/app/theme.dart`. These five anchors
show the OKLCH authoring coordinates (`L C h`) and their committed sRGB values:

| Palette | Mode | Primary | Secondary | Accent | Surface | On surface |
|---|---|---|---|---|---|---|
| Aizome · linen | light | `0.373 0.059 253.0` / `#29425F` | `0.449 0.033 248.6` / `#475767` | `0.496 0.126 42.4` / `#9B451F` | `0.951 0.021 85.9` / `#F5EEDF` | `0.293 0.020 251.5` / `#252D36` |
| Aizome · linen | dark | `0.866 0.030 251.3` / `#C5D5E7` | `0.825 0.015 244.8` / `#BEC7CF` | `0.828 0.117 59.3` / `#FFB477` | `0.232 0.020 254.9` / `#171E27` | `0.924 0.021 81.8` / `#EDE5D7` |
| Navy · cream | light | `0.347 0.073 256.7` / `#1F3A5F` | `0.458 0.032 251.9` / `#4B5969` | `0.480 0.103 71.5` / `#815200` | `0.956 0.024 88.2` / `#F7F0DF` | `0.290 0.028 255.2` / `#222C39` |
| Navy · cream | dark | `0.855 0.045 256.8` / `#BDD1ED` | `0.835 0.018 253.4` / `#C1CAD5` | `0.814 0.126 73.4` / `#F3B55F` | `0.220 0.026 258.3` / `#131B27` | `0.930 0.025 83.4` / `#F0E7D6` |
| Sumi · washi | light | `0.333 0.004 264.5` / `#353638` | `0.439 0.010 84.6` / `#55524C` | `0.478 0.120 41.5` / `#93411F` | `0.950 0.013 86.8` / `#F2EEE5` | `0.277 0.004 84.6` / `#292826` |
| Sumi · washi | dark | `0.859 0.013 82.4` / `#D5D0C7` | `0.822 0.015 84.6` / `#C9C4BA` | `0.774 0.118 54.1` / `#F0A06A` | `0.227 0.004 84.6` / `#1D1C1A` | `0.921 0.015 80.7` / `#EAE4DA` |
| Evening kitchen | light | `0.332 0.076 276.4` / `#2C315C` | `0.459 0.034 280.7` / `#54566B` | `0.480 0.103 71.5` / `#815200` | `0.932 0.023 87.2` / `#EFE8D8` | `0.288 0.031 281.6` / `#28293A` |
| Evening kitchen | dark | `0.888 0.056 283.0` / `#D4D6FF` | `0.840 0.028 288.1` / `#C9C8DC` | `0.814 0.126 73.4` / `#F3B55F` | `0.200 0.038 277.8` / `#121427` | `0.924 0.023 84.6` / `#EDE5D5` |
| Tea garden | light | `0.406 0.046 154.2` / `#35513E` | `0.459 0.025 151.7` / `#4E5C51` | `0.486 0.100 52.1` / `#8B4D23` | `0.952 0.018 89.4` / `#F4EFE2` | `0.298 0.018 151.8` / `#273029` |
| Tea garden | dark | `0.865 0.036 151.6` / `#C2DAC7` | `0.839 0.019 150.5` / `#C2CEC4` | `0.793 0.111 59.9` / `#F0AA70` | `0.221 0.015 157.5` / `#151D18` | `0.927 0.015 94.2` / `#EAE7DC` |
| Rainy veranda | light | `0.425 0.049 241.7` / `#365267` | `0.460 0.022 238.2` / `#4D5A63` | `0.503 0.101 47.2` / `#92502D` | `0.951 0.015 80.7` / `#F4EEE4` | `0.303 0.022 243.2` / `#253039` |
| Rainy veranda | dark | `0.867 0.032 236.4` / `#C0D7E6` | `0.837 0.013 236.7` / `#C2CBD1` | `0.801 0.103 57.0` / `#F1AD7B` | `0.222 0.016 244.4` / `#151C22` | `0.927 0.015 80.7` / `#ECE6DC` |
| Cedar · clay | light | `0.402 0.027 46.6` / `#55443C` | `0.453 0.021 50.1` / `#60534C` | `0.477 0.104 50.5` / `#8A491F` | `0.951 0.014 78.3` / `#F4EEE5` | `0.287 0.013 51.7` / `#302925` |
| Cedar · clay | dark | `0.849 0.028 53.1` / `#DDC9BD` | `0.828 0.017 56.1` / `#D0C4BC` | `0.786 0.117 55.8` / `#F3A56D` | `0.224 0.011 48.2` / `#201A17` | `0.924 0.014 60.6` / `#EDE4DD` |
