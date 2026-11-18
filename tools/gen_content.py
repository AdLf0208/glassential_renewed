#!/usr/bin/env python3
"""Generate the Glassential Renewed slice-1 content tree from upstream.

Slice 1 is the 24 full-cube glasses: the ethereal/ghostly/dark set, luminous
and redstone glass, smart and colour-able glass, stone/sandstone/ice/iron/
obsidian/gravity glass, the two clear-fluid glasses and the four lamps. The
one-way glasses are deferred (custom baked model).

The models, blockstates, textures and `.mcmeta` files are copied byte for
byte from the upstream 1.21.1 checkout; Horizon reads Fusion connected
textures (`crates/horizon-client-render/src/fusion.rs`), so the CTM sheets and
their metadata stay as the author wrote them. What is rewritten:

- `data/glassential/recipe/*.json` from the 1.21.1 ingredient objects to the
  26.3 string form (`{"item": X}` -> `"X"`, `{"tag": T}` -> `"#T"`), the same
  upgrade the author's own 26.3 branch applies;
- loot tables in 26.3's one-condition pool form, `dropWhenSilkTouch` for every
  glass (`GlassentialBlockLoot.java:19-31`);
- `assets/glassential/items/<id>.json` item definitions (26.3 moved them out
  of `models/item`);
- `data/<id>/horizon/block/*.json` template declarations, the two creative
  tabs and the block tags.

Usage:
    python3 tools/gen_content.py [--upstream DIR] [--out DIR]
"""

from __future__ import annotations

import argparse
import json
import shutil
from pathlib import Path

from PIL import Image

# id, template, light, tinted (GLASSES_TINTED), states, tab, extra properties
# `states` is 1 unless the block declares boolean properties: the two
# colour-able glasses and smart glass declare one, the two clear-fluid glasses
# declare six (BigGlassBlockEntity.java:12-70).
VARIANTS = [
    ("glass_dark_ethereal", "minecraft:tinted_glass", 0, False, 1, "functional", {}),
    ("glass_dark_ethereal_reverse", "minecraft:tinted_glass", 0, False, 1, "functional", {}),
    ("glass_ethereal", "minecraft:glass", 0, False, 1, "functional", {}),
    ("glass_ethereal_reverse", "minecraft:glass", 0, False, 1, "functional", {}),
    ("glass_ghostly", "minecraft:glass", 0, False, 1, "functional", {}),
    ("colorable_glass", "minecraft:glass", 0, False, 2, "functional", {}),
    ("colorable_stained_glass", "minecraft:white_stained_glass", 0, False, 2, "functional", {}),
    ("glass_light", "minecraft:glass", 15, False, 1, "functional", {}),
    ("glass_light_tinted", "minecraft:tinted_glass", 15, True, 1, "functional", {}),
    ("glass_redstone", "minecraft:glass", 0, False, 1, "functional", {}),
    ("glass_redstone_tinted", "minecraft:tinted_glass", 0, True, 1, "functional", {}),
    ("smart_glass", "minecraft:glass", 0, False, 2, "functional", {}),
    (
        "obsidian_glass",
        "minecraft:glass",
        0,
        False,
        1,
        "functional",
        {"hardness": 1.0, "explosion_resistance": 1200.0, "requires_tool": True},
    ),
    ("stone_glass", "minecraft:glass", 0, False, 1, "blocks", {}),
    ("sandstone_glass", "minecraft:glass", 0, False, 1, "blocks", {}),
    ("ice_glass", "minecraft:glass", 0, False, 1, "blocks", {}),
    ("iron_glass", "minecraft:glass", 0, False, 1, "blocks", {}),
    ("gravity_glass", "minecraft:glass", 0, False, 1, "functional", {}),
    (
        "clear_fluid_glass",
        "minecraft:glass",
        0,
        False,
        64,
        "functional",
        {"hardness": 0.6, "explosion_resistance": 0.6},
    ),
    (
        "clear_fluid_fake_glass",
        "minecraft:glass",
        0,
        False,
        64,
        "functional",
        {"hardness": 0.6, "explosion_resistance": 0.6},
    ),
    ("glass_lava_lamp", "minecraft:glass", 15, False, 1, "blocks", {}),
    ("glass_lava_lamp_tinted", "minecraft:tinted_glass", 15, True, 1, "blocks", {}),
    ("glass_glowstone_lamp", "minecraft:glass", 15, False, 1, "blocks", {}),
    ("glass_glowstone_lamp_tinted", "minecraft:tinted_glass", 15, True, 1, "blocks", {}),
]

# The state properties each block declares beside its template's
# (`GlassentialBlocks.java` / the block classes).
STATE_PROPERTIES = {
    "colorable_glass": {"lit": {"type": "boolean"}},
    "colorable_stained_glass": {"lit": {"type": "boolean"}},
    "smart_glass": {"powered": {"type": "boolean"}},
    "clear_fluid_glass": {
        name: {"type": "boolean"} for name in ("up", "down", "east", "west", "north", "south")
    },
    "clear_fluid_fake_glass": {
        name: {"type": "boolean"} for name in ("up", "down", "east", "west", "north", "south")
    },
}

TAB_TITLES = {
    "functional": "Glassential Renewed - Functional",
    "blocks": "Glassential Renewed - Blocks",
}
TAB_ICONS = {"functional": "glassential:glass_dark_ethereal", "blocks": "glassential:stone_glass"}

LAMPS = {
    "glass_lava_lamp",
    "glass_lava_lamp_tinted",
    "glass_glowstone_lamp",
    "glass_glowstone_lamp_tinted",
}
CLEAR_FLUID = {"clear_fluid_glass", "clear_fluid_fake_glass"}
# Upstream marks obsidian glass `requiresCorrectToolForDrops`
# (`GlassentialBlocks.java:196-198`) but its datagen never puts it in a
# mineable tag, so no tool is "correct". The port adds the pickaxe tag so the
# declared requirement is usable; recorded as a datagen deviation.
PICKAXE = {"obsidian_glass"}


def crop_tile(source: Path, target: Path) -> None:
    """The plain top-left 16x16 tile of a Fusion sheet, as an item icon."""
    target.parent.mkdir(parents=True, exist_ok=True)
    with Image.open(source) as image:
        image.crop((0, 0, 16, 16)).save(target)


def cube_all(sprite: str, force_translucent: bool) -> dict:
    value: dict[str, object] = {"sprite": sprite}
    if force_translucent:
        value["force_translucent"] = True
    return {"parent": "minecraft:block/cube_all", "textures": {"all": value}}


def render_type(assets: Path, name: str) -> str | None:
    """A texture's Fusion `render_type`, or None."""
    meta = assets / "textures" / f"{name}.png.mcmeta"
    if not meta.is_file():
        return None
    return json.loads(meta.read_text()).get("fusion", {}).get("render_type")


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n")


def upgrade_recipe(value: dict) -> dict:
    """1.21.1 ingredient objects -> 26.3 ingredient strings."""

    def ingredient(entry):
        if isinstance(entry, str):
            return entry
        if "item" in entry:
            return entry["item"]
        if "tag" in entry:
            return "#" + entry["tag"]
        raise SystemExit(f"unknown ingredient shape: {entry!r}")

    out = dict(value)
    if "key" in out:
        out["key"] = {k: ingredient(v) for k, v in out["key"].items()}
    if "ingredients" in out:
        out["ingredients"] = [ingredient(v) for v in out["ingredients"]]
    return out


def model_names(value: object) -> set[str]:
    """`glassential:` model names a blockstate or item model refers to."""
    found: set[str] = set()

    def walk(node: object) -> None:
        if isinstance(node, dict):
            for key, child in node.items():
                if key in ("model", "parent") and isinstance(child, str) and child.startswith("glassential:"):
                    found.add(child.split(":", 1)[1])
                elif isinstance(child, (dict, list)):
                    walk(child)
        elif isinstance(node, list):
            for child in node:
                walk(child)

    walk(value)
    return found


def referenced_textures(model: dict) -> set[str]:
    """`glassential:block/...` texture names a model's textures name."""
    found: set[str] = set()

    def add(value: object) -> None:
        if isinstance(value, str) and value.startswith("glassential:"):
            found.add(value.split(":", 1)[1])
        elif isinstance(value, dict):
            add(value.get("sprite"))

    textures = model.get("textures")
    if isinstance(textures, dict):
        for value in textures.values():
            add(value)
    for element in model.get("elements", []):
        for face in element.get("faces", {}).values():
            add(face.get("texture"))
    return found


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--upstream",
        type=Path,
        default=Path(__file__).resolve().parents[2] / "_shared/upstream/glassential",
    )
    # The package root: `assets/` and `data/` are the carried content of the flat `.hmod`
    # (the retired layout wrote them under `content/`). It does not write `horizon-mod.toml`;
    # that manifest is hand-maintained now.
    parser.add_argument("--out", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()

    upstream = args.upstream / "src/main/resources"
    assets = upstream / "assets/glassential"
    recipes = upstream / "data/glassential/recipe"
    out = args.out
    if not (assets / "blockstates").is_dir():
        raise SystemExit(f"no upstream assets under {assets}")
    if out.exists():
        for sub in ("assets", "data"):
            shutil.rmtree(out / sub, ignore_errors=True)

    # ------------------------------------------------------------- assets
    # Blockstates first, then every model they name (recursively through
    # `parent`), then an item model per glass.
    models: set[str] = set()
    copied_models: list[Path] = []
    needed_textures: set[str] = set()
    for vid, *_rest in VARIANTS:
        source = assets / "blockstates" / f"{vid}.json"
        target = out / "assets/glassential/blockstates" / f"{vid}.json"
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
        models |= model_names(json.loads(source.read_text()))
        write_json(
            out / "assets/glassential/items" / f"{vid}.json",
            {"model": {"type": "minecraft:model", "model": f"glassential:item/{vid}"}},
        )
        item_target = out / "assets/glassential/models/item" / f"{vid}.json"
        item_target.parent.mkdir(parents=True, exist_ok=True)
        if vid in LAMPS:
            # The lamp model has real elements and vanilla glass/glowstone
            # textures: its item is the block model.
            item_source = assets / "models/item" / f"{vid}.json"
            shutil.copyfile(item_source, item_target)
            copied_models.append(item_target)
            models |= model_names(json.loads(item_source.read_text()))
            continue
        if vid in CLEAR_FLUID:
            sprite = "block/clear_fluid_fake_glass_item" if "fake" in vid else "block/clear_fluid_glass_item"
            needed_textures.add(sprite)
            force = True
        else:
            # The block model is a Fusion sheet: the inventory icon is its
            # plain top-left tile, cropped to 16x16 (a Fusion layout's first
            # tile is the unconnected face).
            block_model = json.loads((assets / "models/block" / f"{vid}.json").read_text())
            sprite = block_model["textures"]["all"]
            if isinstance(sprite, dict):
                sprite = sprite["sprite"]
            name = sprite.split(":", 1)[1]
            needed_textures.add(name)
            crop_tile(
                assets / "textures" / f"{name}.png",
                out / "assets/glassential/textures/block" / f"{vid}_item.png",
            )
            sprite = f"block/{vid}_item"
            force = render_type(assets, name) != "cutout"
        write_json(item_target, cube_all(sprite, force))
    while models:
        name = models.pop()
        target = out / "assets/glassential/models" / f"{name}.json"
        if target.exists():
            continue
        source = assets / "models" / f"{name}.json"
        if not source.is_file():
            raise SystemExit(f"model names a missing file: {source}")
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
        copied_models.append(target)
        models |= model_names(json.loads(source.read_text()))

    # Every texture the copied upstream models name, plus the generated item
    # icons' sources, with the Fusion `.mcmeta` when the upstream has one.
    textures: set[str] = set(needed_textures)
    for model_path in copied_models:
        textures |= referenced_textures(json.loads(model_path.read_text()))
    for texture in sorted(textures):
        source = assets / "textures" / f"{texture}.png"
        if not source.is_file():
            raise SystemExit(f"model names a missing texture: {source}")
        target = out / "assets/glassential/textures" / f"{texture}.png"
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
        meta = source.with_name(source.name + ".mcmeta")
        if meta.is_file():
            shutil.copyfile(meta, target.with_name(target.name + ".mcmeta"))

    # -------------------------------------------------------------- data
    total_states = 0
    for vid, template, light, _tinted, states, _tab, props in VARIANTS:
        declaration: dict[str, object] = {"template": template}
        properties = dict(props)
        if light:
            properties["light"] = light
        if properties:
            declaration["properties"] = properties
        if vid in STATE_PROPERTIES:
            declaration["state_properties"] = STATE_PROPERTIES[vid]
        declaration["item"] = True
        write_json(out / "data/glassential/horizon/block" / f"{vid}.json", declaration)
        total_states += states

        write_json(
            out / "data/glassential/loot_table/blocks" / f"{vid}.json",
            {
                "type": "minecraft:block",
                "pools": [
                    {
                        "condition": "minecraft:tool/can_silk_touch",
                        "entries": [{"type": "minecraft:item", "name": f"glassential:{vid}"}],
                        "rolls": 1,
                    }
                ],
                "random_sequence": f"glassential:blocks/{vid}",
            },
        )
        write_json(
            out / "data/glassential/recipe" / f"{vid}.json",
            upgrade_recipe(json.loads((recipes / f"{vid}.json").read_text())),
        )
    if total_states != 153:
        raise SystemExit(f"state total {total_states} != 153")

    ids = [v[0] for v in VARIANTS]
    write_json(
        out / "data/minecraft/tags/block/impermeable.json",
        {"values": [f"glassential:{vid}" for vid in ids]},
    )
    write_json(
        out / "data/c/tags/block/glass_blocks.json",
        {"values": [f"glassential:{vid}" for vid in ids]},
    )
    write_json(
        out / "data/c/tags/block/glass_blocks/tinted.json",
        {"values": [f"glassential:{v[0]}" for v in VARIANTS if v[3]]},
    )
    write_json(
        out / "data/minecraft/tags/block/mineable/pickaxe.json",
        {"values": [f"glassential:{v[0]}" for v in VARIANTS if v[0] in PICKAXE]},
    )
    for tab, suffix in (("functional", ""), ("blocks", "_misc")):
        items = [f"glassential:{v[0]}" for v in VARIANTS if v[5] == tab]
        write_json(
            out / "data/glassential/horizon/creative_tab" / f"glassential_tab{suffix}.json",
            {"title": TAB_TITLES[tab], "icon": TAB_ICONS[tab], "items": items},
        )

    # -------------------------------------------------------------- lang
    upstream_lang = json.loads((assets / "lang/en_us.json").read_text())
    lang = {f"block.glassential.{vid}": upstream_lang[f"block.glassential.{vid}"] for vid in ids}
    lang["itemGroup.glassential"] = TAB_TITLES["functional"]
    lang["itemGroup.glassential_misc"] = TAB_TITLES["blocks"]
    write_json(out / "assets/glassential/lang/en_us.json", lang)

    files = sum(1 for p in out.rglob("*") if p.is_file())
    print(
        f"{len(VARIANTS)} glasses, {total_states} states, {len(textures)} textures, "
        f"{files} files under {out}"
    )


if __name__ == "__main__":
    main()
