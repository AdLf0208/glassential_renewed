#!/usr/bin/env python3
"""Craft the Glassential Renewed recipes in a real crafting table.

Called by tools/e2e.sh with the open game's control socket. Uses the
harness's crafting grid (`ctl screen`: slots 100-108 row-major, result 200)
and asserts, against the upstream 3.4.7 recipes:

1. eight `minecraft:glass` around one `minecraft:glowstone` -> eight
   `glassential:glass_light` (shaped), and the stack reaches the inventory;
2. eight `glassential:glass_light` around one `minecraft:tinted_glass` ->
   eight `glassential:glass_light_tinted` (shaped);
3. four `minecraft:glass` + one `minecraft:ender_eye` -> four
   `glassential:glass_ethereal` (shapeless);
4. one `glassential:glass_dark_ethereal` -> one
   `glassential:glass_dark_ethereal_reverse` (shapeless);
5. the stick controls leave the result empty.

Exits 1 with the observed result on the first mismatch.
"""
import json
import subprocess
import sys

RING = [100, 101, 102, 103, 105, 106, 107, 108]
CENTRE = 104
FIRST_FIVE = [100, 101, 102, 103, 104]


def main():
    sock, ctl, x, y, z = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5]

    def run(*args):
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        try:
            return json.loads(out.stdout)
        except json.JSONDecodeError:
            raise SystemExit(f"ctl {' '.join(args)}: {out.stdout}{out.stderr}")

    def wait(ticks=4):
        run("wait", str(ticks))

    def slots():
        screen = run("screen")
        if "slots" not in screen:
            raise SystemExit(f"no menu is open: {screen}")
        return screen["slots"]

    def find(item):
        for slot in slots():
            if slot.get("item") == item:
                return slot["slot"]
        raise SystemExit(f"missing ingredient {item} in the inventory")

    def result():
        stack = [s for s in slots() if s["slot"] == 200]
        if not stack or not stack[0].get("item"):
            return None
        return stack[0]

    def place(item, mode, targets):
        slot = find(item)
        run("click-slot", str(slot))
        wait(1)
        for target in targets:
            if mode == "right":
                run("click-slot", str(target), "right")
            else:
                run("click-slot", str(target))
            wait(1)
        run("click-slot", str(slot))
        wait(2)

    def setup(items):
        try:
            run("close")
        except SystemExit:
            pass
        wait(5)
        run("setblock", x, y, z, "minecraft:air")
        run("setblock", x, y, z, "minecraft:crafting_table")
        wait(2)
        run("command", f"tp @s {float(x) - 0.5} {y} {float(z) + 0.5}")
        wait(3)
        run("command", "clear @s")
        for item, count in items:
            run("give", item, str(count))
        wait(3)
        run("interact", x, y, z)
        wait(5)

    failures = []

    def check(label, items, placements, want_item, want_count):
        setup(items)
        for item, mode, targets in placements:
            place(item, mode, targets)
        got = result()
        print(f"{label} ->", json.dumps(got))
        if not got or got.get("item") != want_item or got.get("count") != want_count:
            failures.append(f"{label}: expected {want_count} {want_item}, got {got}")
            return None
        return got

    def empty_control(label, items, placements):
        setup(items)
        for item, mode, targets in placements:
            place(item, mode, targets)
        got = result()
        print(f"{label} ->", json.dumps(got))
        if got is not None:
            failures.append(f"{label}: control crafted {got['item']}")

    # 1. Luminous glass, and the crafted stack reaches the inventory.
    got = check(
        "8 glass + glowstone",
        [("minecraft:glass", 8), ("minecraft:glowstone", 1)],
        [
            ("minecraft:glass", "right", [str(slot) for slot in RING]),
            ("minecraft:glowstone", "right", [str(CENTRE)]),
        ],
        "glassential:glass_light",
        8,
    )
    if got:
        run("click-slot", "200")
        wait(5)
        run("click-slot", "9")
        wait(3)
        made = run("inventory")["counts"].get("glassential:glass_light", 0)
        if made < 8:
            failures.append(f"the crafted glass reached the inventory as {made}, not 8")

    empty_control(
        "8 glass + stick",
        [("minecraft:glass", 8), ("minecraft:stick", 1)],
        [
            ("minecraft:glass", "right", [str(slot) for slot in RING]),
            ("minecraft:stick", "right", [str(CENTRE)]),
        ],
    )

    # 2. Tinted luminous glass from luminous glass around tinted glass.
    check(
        "8 glass_light + tinted_glass",
        [("glassential:glass_light", 8), ("minecraft:tinted_glass", 1)],
        [
            ("glassential:glass_light", "right", [str(slot) for slot in RING]),
            ("minecraft:tinted_glass", "right", [str(CENTRE)]),
        ],
        "glassential:glass_light_tinted",
        8,
    )
    empty_control(
        "8 glass_light + stick",
        [("glassential:glass_light", 8), ("minecraft:stick", 1)],
        [
            ("glassential:glass_light", "right", [str(slot) for slot in RING]),
            ("minecraft:stick", "right", [str(CENTRE)]),
        ],
    )

    # 3. Ethereal glass, shapeless.
    check(
        "4 glass + ender eye",
        [("minecraft:glass", 4), ("minecraft:ender_eye", 1)],
        [
            ("minecraft:glass", "right", [str(slot) for slot in FIRST_FIVE[:4]]),
            ("minecraft:ender_eye", "right", [str(CENTRE)]),
        ],
        "glassential:glass_ethereal",
        4,
    )
    empty_control(
        "4 glass + stick",
        [("minecraft:glass", 4), ("minecraft:stick", 1)],
        [
            ("minecraft:glass", "right", [str(slot) for slot in FIRST_FIVE[:4]]),
            ("minecraft:stick", "right", [str(CENTRE)]),
        ],
    )

    # 4. Reverse tinted ethereal glass, shapeless one-for-one.
    check(
        "1 glass_dark_ethereal",
        [("glassential:glass_dark_ethereal", 1)],
        [("glassential:glass_dark_ethereal", "right", [str(FIRST_FIVE[0])])],
        "glassential:glass_dark_ethereal_reverse",
        1,
    )
    empty_control(
        "1 stick",
        [("minecraft:stick", 1)],
        [("minecraft:stick", "right", [str(FIRST_FIVE[0])])],
    )

    run("close")
    wait(3)
    if failures:
        for failure in failures:
            print(f"FAIL: {failure}", file=sys.stderr)
        raise SystemExit(1)
    print("recipe PASS: luminous, tinted luminous, ethereal and reverse recipes craft, and the stick controls stay empty")


if __name__ == "__main__":
    main()
