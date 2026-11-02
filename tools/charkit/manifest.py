"""Which LPC parts the kit ships: (id, sheet dir under spritesheets/, material, z).

z follows the generator's zPos: body 10, legs 20, torso 35, head 100, hair 120, hat 130, weapon 145.
"""
ANIMS = ('walk', 'idle')

# The palette each material's sheets are drawn in (palette_definitions/*/meta_*.json "base").
MATERIAL_BASE = {'body': 'light', 'hair': 'orange', 'cloth': 'white', 'metal': 'steel'}

MANIFEST = [
    # bodies and heads
    ('body_male', 'body/bodies/male', 'body', 10),
    ('body_female', 'body/bodies/female', 'body', 10),
    ('body_muscular', 'body/bodies/muscular', 'body', 10),
    ('head_male', 'head/heads/human/male', 'body', 100),
    ('head_female', 'head/heads/human/female', 'body', 100),
    ('head_male_elderly', 'head/heads/human/male_elderly', 'body', 100),
    ('head_female_elderly', 'head/heads/human/female_elderly', 'body', 100),
    ('ears_elven', 'head/ears/elven/adult', 'body', 105),
    ('ears_cat', 'head/ears/cat/skin/adult', 'body', 131),
    ('tail_cat', 'body/tail/cat/adult', 'body', 65),
    ('tail_fluffy', 'body/tail/fluffy/adult', 'hair', 65),
    ('tail_wolf', 'body/tail/wolf/adult', 'hair', 65),
    # hair
    ('hair_plain', 'hair/plain/adult', 'hair', 120),
    ('hair_long', 'hair/long/adult', 'hair', 120),
    ('hair_ponytail', 'hair/ponytail/adult', 'hair', 120),
    ('hair_bangs_bun', 'hair/bangs_bun/adult', 'hair', 120),
    ('hair_spiked', 'hair/spiked/adult', 'hair', 120),
    ('hair_bob', 'hair/bob/adult', 'hair', 120),
    ('hair_pigtails', 'hair/pigtails/adult', 'hair', 120),
    ('hair_swoop', 'hair/swoop/adult', 'hair', 120),
    ('hair_princess', 'hair/princess/adult', 'hair', 120),
    ('hair_curls_large', 'hair/curls_large/adult', 'hair', 120),
    ('hair_braid2', 'hair/braid2/adult', 'hair', 120),
    ('hair_braid', 'hair/braid/adult', 'hair', 120),
    ('hair_long_straight', 'hair/long_straight/adult', 'hair', 120),
    ('hair_messy1', 'hair/messy1/adult', 'hair', 120),
    ('hair_long_tied', 'hair/long_tied/adult', 'hair', 120),
    ('hair_parted', 'hair/parted/adult', 'hair', 120),
    ('hair_high_and_tight', 'hair/high_and_tight/adult', 'hair', 120),
    ('hair_high_ponytail', 'hair/high_ponytail/adult', 'hair', 120),
    ('hair_bangs', 'hair/bangs/adult', 'hair', 120),
    # torso
    ('longsleeve_male', 'torso/clothes/longsleeve/longsleeve/male', 'cloth', 35),
    ('longsleeve_female', 'torso/clothes/longsleeve/longsleeve/female', 'cloth', 35),
    ('blouse_female', 'torso/clothes/blouse/female', 'cloth', 35),
    ('robe_female', 'torso/clothes/robe/female', 'cloth', 35),
    ('vest_male', 'torso/clothes/vest/male', 'cloth', 36),
    ('plate_male', 'torso/armour/plate/male', 'metal', 35),
    ('plate_female', 'torso/armour/plate/female', 'metal', 35),
    ('leather_male', 'torso/armour/leather/male', 'cloth', 35),
    ('leather_female', 'torso/armour/leather/female', 'cloth', 35),
    ('apron_full_female', 'torso/aprons/apron_full/female', 'cloth', 40),
    ('apron_male', 'torso/aprons/apron/male', 'cloth', 40),
    ('dress_slit_female', 'dress/slit/female', 'cloth', 33),
    ('sash_female', 'dress/sash/female', 'cloth', 41),
    # legs and feet
    ('pants_male', 'legs/pants/male', 'cloth', 20),
    ('pants_female', 'legs/pants/thin', 'cloth', 20),
    ('skirt_female', 'legs/skirts/plain/thin', 'cloth', 22),
    ('skirt_male', 'legs/skirts/plain/male', 'cloth', 22),
    ('shorts_female', 'legs/shorts/shorts/thin', 'cloth', 20),
    ('shorts_male', 'legs/shorts/shorts/male', 'cloth', 20),
    ('legplate_female', 'legs/armour/plate/thin', 'metal', 21),
    ('legplate_male', 'legs/armour/plate/male', 'metal', 21),
    ('boots_female', 'feet/boots/basic/thin', 'cloth', 15),
    ('boots_male', 'feet/boots/basic/male', 'cloth', 15),
    ('shoes_female', 'feet/shoes/basic/thin', 'cloth', 15),
    ('shoes_male', 'feet/shoes/basic/male', 'cloth', 15),
    ('footplate_female', 'feet/armour/plate/thin', 'metal', 16),
    ('footplate_male', 'feet/armour/plate/male', 'metal', 16),
    ('pauldrons_female', 'shoulders/pauldrons/thin', 'metal', 60),
    ('pauldrons_male', 'shoulders/pauldrons/male', 'metal', 60),
    # hats
    ('hat_wizard', 'hat/magic/wizard/base/adult', 'cloth', 130),
    ('hat_bandana', 'hat/cloth/bandana/adult', 'cloth', 130),
    ('hat_hood', 'hat/cloth/hood/adult', 'cloth', 130),
    ('hat_leather_cap', 'hat/cloth/leather_cap/adult', 'cloth', 130),
    ('hat_headband', 'hat/headband/thick/adult', 'cloth', 130),
    ('hat_kettle', 'hat/helmet/kettle/adult', 'metal', 130),
    ('hat_tiara', 'hat/formal/tiara/adult', 'metal', 130),
    # held
    ('sword_longsword', 'weapon/sword/longsword', None, 145),
    ('sword_dagger', 'weapon/sword/dagger', None, 145),
    ('staff_simple', 'weapon/magic/simple/foreground', None, 145),
    ('staff_simple_bg', 'weapon/magic/simple/background', None, 5),
]
