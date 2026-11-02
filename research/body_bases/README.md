# Body bases: can LPC or SpriteKiln be customised to our look? (2026-11-02)

Target look: anime 3/4 top-down, chibi-ish proportions (head ~1/3 of height), visible bust with jiggle,
expressive hand-drawn anime eyes/lips, 64x128 cells, recolourable skin/hair/cloth.

Evidence: `.context/wip/sk_vs_angeler.png` (Angeler base vs SpriteKiln chibi test at size 128 and 64),
`.context/wip/old_lpc/` (LPC body renders rejected 2026-11-02), `sk_chibi_test.yaml` (the test spec).

## LPC (Liberated Pixel Cup / Universal LPC Spritesheet Generator)
- What it is: hand-drawn 64x64 PNG sheets, one per body type x animation x direction x frame. Customisation
  = pick parts (body type, hair, clothes) + palette recolour. No sliders, no skeleton, no proportion knobs.
- Body types shipped: male, female, muscular, pregnant, teen, child (all Western semi-realistic proportions,
  head ~1/5 height, flat chest even on female).
- To get our look you must redraw: the body for every frame of every animation in 4 directions (walk 9,
  idle, run, slash, cast, thrust, shoot, hurt ... hundreds of frames), AND every clothing/hair sheet,
  because LPC clothes are drawn to the LPC silhouette. Nothing from the ecosystem survives a proportion
  change except the face overlay (which already worked: our 128px anime eyes).
- Verdict: not customisable in the sense asked. It is a parts library, not a generator. Changing the body
  = throwing the library away. Already rejected by user ("don't look human").

## SpriteKiln v0.3 (MIT, procedural 3D YAML -> pixel art)
- What it is: SDF skeleton + primitives + materials, rendered to 8-direction pixel art. Fully parametric:
  `extends: humanoid`, override bones/parts/animations; head scale, bust ellipsoids, jiggle channel, toon
  shading, dark outline all took ~40 lines of YAML (sk_chibi_test.yaml). Pose render 1.6s at 64, 4.9s at
  128 (numpy backend).
- Test result (sk_vs_angeler.png): body proportions, bust and long hair DO come out. But the look is a
  pixelised 3D doll: skinny tube limbs, hard volumetric shading, no face (eyes are 1-2px `ink` prims
  hidden under the fringe), side view thin and lumpy, hair is blocky SDF locks. No 2D decals, no
  hand-drawn eyes/mouth, no soft anime shading (`surface paint` = stripes/rings/spots only).
- Its own docs say faces are the weak point ("no nose by default ... at 10 px a nose is a lump") and the
  showcase is faceless realistic-proportion fantasy (Templar, Huntress).
- Getting to anime quality would mean: SK body + hand-drawn face/hair overlays per direction per frame
  (face follows head bone -> need a projection step we would write), plus hand-cleanup of limbs. More
  work than drawing on the Angeler base, with a worse starting point.
- Verdict: customisable yes, right look no. Useful later for things that are NOT anime faces: monsters,
  armoured knights with helmets, props/weapons (sword/axe/shield/bow kits), 8-direction turnarounds for
  reference. Not the character base.

## Angeler "Basic Character Ver 64" (current base)
- Hand-drawn, already the exact look: anime face, bust + jiggle baked into idle/walk, soft shading,
  64x128, 4 dirs, idle 8 / walk 10 / run 8. Split into recolourable layers by tools/charkit64/extract.py.
- Customisation cost: new hair / outfit / expression = draw at this scale on top of the base (per dir,
  per frame where it moves). Male body = edit the female sheets (flatten bust, widen shoulders).
- Licence: commercial + modify OK, no redistribution -> assets/charkit64 gitignored, rebuilt locally.
- Verdict: keep. It is the only option where the starting point already passes the user's eye.

## Decision
Keep Angeler base. Drop LPC for bodies (keep nothing but the lesson: face overlay technique). Keep
SpriteKiln in the back pocket for monsters/props/helmeted enemies; revisit only if a decal/face feature
lands upstream.

## Status (2026-11-02, user)
Angeler cast render (`.context/wip/base64_cast.png`, local only: licence forbids redistribution) judged
"alright, needs polish". LPC kit (`tools/charkit`, `assets/charkit`) and this SpriteKiln test stay as live
experiments; final pick per thing (bodies, monsters, props) decided by results, not upfront.

Files here: `sk_chibi_test.yaml` (run: `python -m spritekiln pose sk_chibi_test.yaml --dirs all` with
SpriteKiln >= 0.3 on PYTHONPATH), `sk_chibi_pose.png`, `sk_chibi128_pose.png`, `sk_humanoid_stock.png`.
Known polish list for the Angeler cast: cloth recolour reads blobby on dark colours (spread 0.55 hack,
outfits will be drawn instead), hero is a female placeholder, all six share one hairstyle, no outfits,
no expressions.

## Angeler store survey (2026-11-02)
Creator page: https://angeler1234.itch.io/ (13 packs). Relevant:
- **Basic Character Ver 64** ($10): bald male + female bases, 64x128, idle/walk/run 8 frames each, 4 dirs,
  no hair/clothes. Our free pack is its female subset (idle 8 / walk 10 / run 8). Male status per comments:
  "unified 8-frame male and female" then "male standard body still planned" -> unclear even if bought.
- **Basic Character Female 64 Free** = what we have (`.context/ref_free/Free/`).
- **Basic Character 48 PRO Free** (48x96, single unlayered NPC, 12-frame idle/walk/run): different scale,
  not layer-split, "not compatible" with their other packs. Reference only. Not downloaded.
- Licence on all: personal + commercial, modify OK, no redistribution/resale, "no AI".

Promo GIFs from the store (user-supplied, `.context/ref/angeler_promo/`): "Standby Show" 4-dir idle of the
bald male and female, and the front pair. GIF is re-encoded (palette differs from the pack, ~0.85x scale,
2px edge runs) so pixels are useless and unlicensed anyway. Use as a **drawing reference** for the male
body (shoulder width, pec/abs line, straight hips, bald scalp shape in all 4 dirs) when editing our female
sheets into a male base. Also confirms their bald head silhouette for our rebuilt scalp.
