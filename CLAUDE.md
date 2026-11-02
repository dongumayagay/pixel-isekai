# Pixel Isekai (fresh start)

2D pixel-art slow-life sandbox sim. Previous attempts archived on branch
`dongumayagay/isekai-pixel-art-rework`; nothing from there is reused.

## Art rules (non-negotiable)
- Hand-drawn pixel art only: ASCII grids per sprite and frame, hand-picked ramps, hand outlines.
- No procedural shading, no 3D-to-pixel, no upscalers, no SDF blobs, no asset packs, no purchases.
- Native scale: 16px tiles, characters ~16x32, like Stardew Valley.
- References are studied by dissection only; never copied, never checked in.
- Every pass ends with an in-game screenshot. Weakest area next.

## Token discipline
- Short replies, no narration, no restating. Code via heredocs, not long Read dumps.
- One render per iteration, small crops at 2-3x. Never dump big sheets to look at a detail.
- Never re-read files already in context. Grep for the lines you need.
- Batch independent tool calls. Decide, don't ask. Report in <=8 lines.
