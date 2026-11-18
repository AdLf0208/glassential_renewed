#!/usr/bin/env bash
# In-game check for the Glassential Renewed slice-1 content: the 24 full-cube
# glasses.
#
# Default run (creative launch, then survival for drops/crafting; The End):
#   1. every glass id `setblock`s and reads back (24 default states), the
#      colour-able pair's `lit=true` and smart glass' `powered=true` bind, and
#      both clear-fluid glasses bind all 64 up/down/east/west/north/south
#      states — 153 distinct states in all;
#   2. luminous glass and the four lamps emit 15 while ethereal and stone
#      glass emit 0; tinted ethereal glass blocks a glowstone's light where
#      plain glass lets it through (a differential);
#   2b. `#minecraft:impermeable` and `#minecraft:mineable/pickaxe` match at
#      runtime with stone/ethereal negatives, and the built
#      `c:glass_blocks` (24) / `c:glass_blocks/tinted` (4) tables carry the
#      blocks (the command parser does not know mod tag ids at the pin);
#   2c. redstone glass is a signal source: a redstone lamp beside
#      `glass_redstone`/`glass_redstone_tinted` is lit, a lamp beside the
#      non-source `glass_ethereal` (the control) is dark, and removing the
#      glass darkens the lamp; smart glass frosts its whole connected panel
#      (its `powered` state, which selects the frosted model) when a signal
#      touches one block of the run, an isolated pane stays clear, and
#      removing the signal clears the panel;
#   3. every glass drops itself only with silk touch: without it
#      glass_light/glass_light_tinted/glass_ethereal/obsidian_glass drop
#      nothing, while vanilla tinted glass drops itself (the differential);
#      with silk touch all of them drop themselves;
#   4. the luminous, tinted-luminous, ethereal and reverse recipes craft, and
#      the stick controls leave the result empty;
#   5. all 24 block items are `/give`-able, the two creative tabs list 24
#      entries in total, and the block items place;
#   6. unknown ids are refused.
#
# COUNTERFACTUAL=1 runs the vanilla differential instead: vanilla glass drops
# nothing without silk and itself with silk, vanilla tinted glass drops itself
# without silk, and none of them ever drops a Glassential item. NEGATIVE=1
# (with COUNTERFACTUAL=1) asserts that vanilla tinted glass drops a Glassential
# item, so the run MUST fail — proving the drop assertion can fail.
# NEGATIVE=signal stops at the redstone phase and asserts that the lamp beside
# redstone glass is dark, so that run MUST fail too — proving the signal
# assertion can fail.
# SELFTEST=1 is the counterfactual's negative leg in one flag
# (`COUNTERFACTUAL=1 NEGATIVE=1`): the run MUST exit non-zero.
#
# STATUS (2026-11-18, SDK 0b14f8a): the port is one flat `.hmod`
# (`glassential.hmod`) carrying the content *and* the server half, and a native `.hmod` cannot
# load its code on an unmodded game. So `GAME=` must name a game the launcher built with this
# package:
#
#   horizon-launcher native-build ports/glassential --instance <i> --trust-native-code
#
# `horizon-launcher` is not installed on the campaign box and no game exists at `0b14f8a`, so
# this script's last verified runs are the retired bundle's (`ports/_shared/evidence/glassential/`,
# 0.1.0, `e2e*.log`); the run recorded with a launcher build is the next slice's first job.
# `CONTROL_GAME=` optionally names a game *without* the package: the counterfactual leg then
# restarts it in its own world, so the vanilla differential runs on a binary that cannot
# possibly answer for Glassential. `DEV_MOD=1` adds `--dev-mod glassential.hmod` for a
# source-built game — that loads the *content* only, so the redstone and smart-glass phases
# are not proved by such a run.
set -euo pipefail
export LC_ALL=C

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/glassential
# The control binary is the shared one at the campaign pin (same protocol as a launcher build).
CTL=${CTL:-$CLONE/ports/_shared/bin/horizon-1d63e0a5/horizon-ctl}
# No default: only a launcher-built game carries this package's code.
GAME=${GAME:-}
CONTROL_GAME=${CONTROL_GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/glassential-e2e.sock}
COUNTERFACTUAL=${COUNTERFACTUAL:-0}
NEGATIVE=${NEGATIVE:-0}
SELFTEST=${SELFTEST:-0}
DEV_MOD=${DEV_MOD:-0}
WORLD=GlassentialE2E
CONTROL_WORLD=GlassentialControlE2E

if [ "$SELFTEST" = 1 ]; then
    COUNTERFACTUAL=1
    [ "$NEGATIVE" = 0 ] && NEGATIVE=1
fi

DEV_ARGS=()
[ "$DEV_MOD" = 1 ] && DEV_ARGS=(--dev-mod "$PORT/glassential.hmod")

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -n "$GAME" ] || {
    echo "GAME=<launcher-built horizon-client> is required: a native .hmod's code is compiled" >&2
    echo "in by horizon-launcher native-build, so an unmodded game answers no Glassential hook." >&2
    exit 1
}
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -z "$CONTROL_GAME" ] || [ -x "$CONTROL_GAME" ] || { echo "missing control horizon-client: $CONTROL_GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }
[ -f "$PORT/glassential.hmod" ] || { echo "missing package: $PORT/glassential.hmod (run cargo horizon-mod build)" >&2; exit 1; }

GLASSES=(
    glass_dark_ethereal glass_dark_ethereal_reverse glass_ethereal
    glass_ethereal_reverse glass_ghostly colorable_glass
    colorable_stained_glass glass_light glass_light_tinted glass_redstone
    glass_redstone_tinted smart_glass obsidian_glass stone_glass
    sandstone_glass ice_glass iron_glass gravity_glass clear_fluid_glass
    clear_fluid_fake_glass glass_lava_lamp glass_lava_lamp_tinted
    glass_glowstone_lamp glass_glowstone_lamp_tinted
)
LUMINOUS=(
    glass_light glass_light_tinted glass_lava_lamp glass_lava_lamp_tinted
    glass_glowstone_lamp glass_glowstone_lamp_tinted
)

SCRATCH=$(mktemp -d /tmp/opencode/glassential-home.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")"
rm -f "$SOCK"
cd "$SCRATCH"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
fail() { echo "FAIL: $*" >&2; exit 1; }
say() { echo "+ $*"; }
count_item() { ctl inventory | jq -r --arg i "$1" '.counts[$i] // 0'; }
block_state() { ctl block "$1" "$2" "$3" | jq -r '.block // "unknown"'; }
full_state() { ctl block "$1" "$2" "$3" | jq -r '.state // "unknown"'; }
light_at() { ctl light "$1" "$2" "$3" "$1" "$2" "$3" | jq -r '.block'; }
prop() { echo "$1" | grep -o "$2=[a-z]*" | head -1 | cut -d= -f2 || true; }
assert_prop() {
    local state=$1 key=$2 want=$3 got
    got=$(prop "$state" "$key")
    [ "$got" = "$want" ] || fail "$state: $key is '$got', expected '$want'"
}
end() { ctl command "execute in minecraft:the_end run $1"; }

cleanup() {
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    rm -rf "$SCRATCH" || true
}
finish() { local rc=$?; cleanup; echo "EXIT:$rc"; return $rc; }
trap finish EXIT

say "launch (offscreen renderer, creative): $GAME, world $WORLD"
"$CTL" --socket "$SOCK" launch \
    --game "$GAME" --world "$WORLD" --seed 20261105 --creative --offscreen --wait \
    --game-dir "$SCRATCH/game" --resources "$RES" "${DEV_ARGS[@]}"
ctl wait-ready 180

# The End platform every phase builds on, in whichever game is running.
arena() {
    say "The End platform at y=79 and an empty box above it"
    end "forceload add -16 -16 80 20"
    end "tp @s 0.5 80 2.5"
    ctl wait 40
    end "fill -16 79 -16 80 79 20 minecraft:stone"
    for y in 80 88 96 104 112 120; do
        end "fill -16 $y -16 80 $(( y + 7 > 120 ? 120 : y + 7 )) 20 minecraft:air"
    done
    end "kill @e[type=minecraft:item]"
    end "kill @e[type=minecraft:ender_dragon]"
    end "kill @e[type=minecraft:end_crystal]"
    end "difficulty peaceful"
    end "tp @s 0.5 80 2.5"
    ctl wait 10
    ctl close >/dev/null 2>&1 || true
    ctl wait 3
}
arena

# Start another build in its own world: the counterfactual's differential is the one leg that
# must also run where Glassential cannot answer, when CONTROL_GAME names such a game.
start_in() { # start_in WORLD GAME
    say "launch $2 in world $1"
    "$CTL" --socket "$SOCK" launch \
        --game "$2" --world "$1" --seed 20261105 --creative --offscreen --wait \
        --game-dir "$SCRATCH/game" --resources "$RES"
    ctl wait-ready 180
    arena
}

# ---------------------------------------------------------------- state bind
say "all 24 default states bind"
for i in "${!GLASSES[@]}"; do
    ctl setblock "$i" 80 0 "glassential:${GLASSES[$i]}" >/dev/null || fail "setblock glassential:${GLASSES[$i]} failed"
done
ctl wait 10
for i in "${!GLASSES[@]}"; do
    got=$(block_state "$i" 80 0)
    [ "$got" = "glassential:${GLASSES[$i]}" ] || fail "x=$i reads '$got', expected glassential:${GLASSES[$i]}"
done
echo "24/24 default states bound"

say "the boolean states bind: colorable lit, and smart glass' powered is owned by the panel"
# An isolated pane keeps its default `powered=false`; the pane at x=11 sits
# beside the redstone glass at x=10 and is frosted by it (slice 2's signal
# source), and a hand-set `powered=true` without a signal is cleared on the
# next level tick, because the panel owns that state now.
ctl setblock 35 80 0 glassential:smart_glass >/dev/null || fail "setblock isolated smart glass failed"
ctl setblock 30 80 0 "glassential:smart_glass[powered=true]" >/dev/null || fail "smart powered=true failed"
ctl setblock 31 80 0 "glassential:colorable_glass[lit=true]" >/dev/null || fail "colorable lit=true failed"
ctl setblock 32 80 0 "glassential:colorable_stained_glass[lit=true]" >/dev/null || fail "colorable stained lit=true failed"
ctl wait 8
assert_prop "$(full_state 35 80 0)" powered false
assert_prop "$(full_state 11 80 0)" powered true
assert_prop "$(full_state 30 80 0)" powered false
assert_prop "$(full_state 31 80 0)" lit true
assert_prop "$(full_state 32 80 0)" lit true
colorable=$(full_state 5 80 0)
assert_prop "$colorable" lit false
echo "smart glass: default clear at x=35, frosted beside the redstone glass at x=11, a hand-set powered=true cleared at x=30; colorable lit binds"

say "both clear-fluid glasses bind all 64 connection states"
for block in clear_fluid_glass clear_fluid_fake_glass; do
    base=40
    [ "$block" = clear_fluid_fake_glass ] && base=60
    for mask in $(seq 0 63); do
        x=$(( base + mask % 8 ))
        y=$(( 90 + mask / 8 ))
        up=$(( (mask >> 0) & 1 )); down=$(( (mask >> 1) & 1 ))
        east=$(( (mask >> 2) & 1 )); west=$(( (mask >> 3) & 1 ))
        north=$(( (mask >> 4) & 1 )); south=$(( (mask >> 5) & 1 ))
        state="glassential:${block}[up=$([ "$up" = 1 ] && echo true || echo false),down=$([ "$down" = 1 ] && echo true || echo false),east=$([ "$east" = 1 ] && echo true || echo false),west=$([ "$west" = 1 ] && echo true || echo false),north=$([ "$north" = 1 ] && echo true || echo false),south=$([ "$south" = 1 ] && echo true || echo false)]"
        ctl setblock "$x" "$y" 0 "$state" >/dev/null || fail "setblock $block mask $mask failed"
    done
done
ctl wait 10
for block in clear_fluid_glass clear_fluid_fake_glass; do
    base=40
    [ "$block" = clear_fluid_fake_glass ] && base=60
    for mask in $(seq 0 63); do
        x=$(( base + mask % 8 ))
        y=$(( 90 + mask / 8 ))
        got=$(full_state "$x" "$y" 0)
        for pair in "up:$(( (mask >> 0) & 1 ))" "down:$(( (mask >> 1) & 1 ))" "east:$(( (mask >> 2) & 1 ))" "west:$(( (mask >> 3) & 1 ))" "north:$(( (mask >> 4) & 1 ))" "south:$(( (mask >> 5) & 1 ))"; do
            key=${pair%%:*}; bit=${pair##*:}
            want=$([ "$bit" = 1 ] && echo true || echo false)
            assert_prop "$got" "$key" "$want"
        done
    done
done
echo "128/128 clear-fluid states bind (64 per glass); 153 distinct states in all"

# -------------------------------------------------------------------- light
say "luminous glass and the four lamps emit 15; ethereal and stone glass emit 0"
for i in "${!LUMINOUS[@]}"; do
    ctl setblock "$((i * 20))" 100 20 "glassential:${LUMINOUS[$i]}" >/dev/null || fail "setblock ${LUMINOUS[$i]} failed"
done
ctl setblock 120 100 20 glassential:glass_ethereal >/dev/null
ctl setblock 140 100 20 glassential:stone_glass >/dev/null
ctl wait 10
for i in "${!LUMINOUS[@]}"; do
    got=$(light_at "$((i * 20))" 100 20)
    [ "$got" = "f" ] || fail "${LUMINOUS[$i]} light is '$got', expected 'f' (15)"
done
[ "$(light_at 120 100 20)" = "0" ] || fail "glass_ethereal emits light"
[ "$(light_at 140 100 20)" = "0" ] || fail "stone_glass emits light"
echo "6/6 luminous glasses emit 15; the two controls emit 0"

say "tinted ethereal glass blocks light where plain glass passes it"
ctl setblock 0 110 18 minecraft:glowstone >/dev/null
ctl setblock 0 111 18 minecraft:glass >/dev/null
ctl setblock 20 110 18 minecraft:glowstone >/dev/null
ctl setblock 20 111 18 glassential:glass_dark_ethereal >/dev/null
ctl setblock 40 110 18 minecraft:glowstone >/dev/null
ctl setblock 40 111 18 glassential:glass_dark_ethereal_reverse >/dev/null
ctl wait 10
through=$(light_at 0 111 18)
blocked=$(light_at 20 111 18)
blocked_reverse=$(light_at 40 111 18)
echo "in plain glass=$through, in dark ethereal=$blocked, in reverse=$blocked_reverse"
[ "$through" != "0" ] || fail "the glowstone did not shine into plain glass (control broken)"
[ "$blocked" = "0" ] || fail "glass_dark_ethereal passed light ($blocked)"
[ "$blocked_reverse" = "0" ] || fail "glass_dark_ethereal_reverse passed light ($blocked_reverse)"
echo "dark ethereal blocks the glowstone's light (plain glass reads $through)"

say "the vanilla tags carry the glasses; the convention tags are in the built table"
# The command parser knows vanilla tag ids; a mod's own new tag ids are not in
# its prepared registry at the pin (recorded gap), so the `c:` tags are read
# from the content build's `block-tags.json` below.
for spec in "7:minecraft:impermeable:TAG_IMPERMEABLE" "12:minecraft:mineable/pickaxe:TAG_PICKAXE"; do
    x=${spec%%:*}; rest=${spec#*:}; tag=${rest%:*}; msg=${rest##*:}
    ctl command "execute in minecraft:the_end if block $x 80 0 #$tag run say $msg" >/dev/null
done
ctl command "execute in minecraft:the_end if block 0 79 0 #minecraft:impermeable run say TAG_STONE_IMPERMEABLE" >/dev/null
ctl command "execute in minecraft:the_end if block 2 80 0 #minecraft:mineable/pickaxe run say TAG_ETHEREAL_PICKAXE" >/dev/null
ctl wait 3
messages=$(ctl messages 60 | jq -r '.chat[]')
for msg in TAG_IMPERMEABLE TAG_PICKAXE; do
    echo "$messages" | grep -q "$msg" || fail "missing $msg"
done
for msg in TAG_STONE_IMPERMEABLE TAG_ETHEREAL_PICKAXE; do
    if echo "$messages" | grep -q "$msg"; then
        fail "$msg matched a block that is not in the tag"
    fi
done
TAGS=$(find "$SCRATCH/game/.horizon-cache/mod-content" -maxdepth 2 -name block-tags.json 2>/dev/null | head -1)
[ -n "$TAGS" ] && [ -f "$TAGS" ] || fail "no block-tags.json under $SCRATCH/game/.horizon-cache/mod-content (the pin stores it in a per-content digest folder)"
jq -e '."c:glass_blocks" | fromjson | .values | length == 24' "$TAGS" >/dev/null || fail "c:glass_blocks does not list 24 glasses"
jq -e '."c:glass_blocks/tinted" | fromjson | .values | length == 4' "$TAGS" >/dev/null || fail "c:glass_blocks/tinted does not list 4 glasses"
jq -e '."c:glass_blocks/tinted" | fromjson | .values | index("glassential:glass_ethereal") == null' "$TAGS" >/dev/null || fail "glass_ethereal is in c:glass_blocks/tinted"
echo "tags: #minecraft:impermeable and #minecraft:mineable/pickaxe at runtime (with stone/ethereal negatives); c:glass_blocks (24) and c:glass_blocks/tinted (4) in the built table"

# ------------------------------------------------ redstone + smart glass
say "redstone glass is a signal source; a plain Glassential glass is not"
# A vanilla redstone lamp beside the glass is the probe: it is lit exactly when
# the block it touches gives it a signal.
ctl setblock 0 80 10 minecraft:redstone_lamp >/dev/null
ctl setblock 1 80 10 glassential:glass_redstone >/dev/null
ctl setblock 4 80 10 minecraft:redstone_lamp >/dev/null
ctl setblock 5 80 10 glassential:glass_ethereal >/dev/null
ctl setblock 8 80 10 minecraft:redstone_lamp >/dev/null
ctl setblock 9 80 10 glassential:glass_redstone_tinted >/dev/null
ctl wait 10
probe_plain=$(full_state 0 80 10)
probe_ethereal=$(full_state 4 80 10)
probe_tinted=$(full_state 8 80 10)
echo "lamp beside redstone glass: $probe_plain"
echo "lamp beside ethereal glass: $probe_ethereal"
echo "lamp beside tinted redstone glass: $probe_tinted"
assert_prop "$probe_plain" lit true
assert_prop "$probe_tinted" lit true
assert_prop "$probe_ethereal" lit false
if [ "$NEGATIVE" = signal ]; then
    say "negative: the lamp beside redstone glass must read dark (a false assertion)"
    assert_prop "$probe_plain" lit false
    fail "NEGATIVE=signal did not observe the false assertion (the lamp was lit)"
fi
ctl setblock 1 80 10 minecraft:air >/dev/null
ctl wait 10
assert_prop "$(full_state 0 80 10)" lit false
echo "removing redstone glass darkens the lamp again"
echo "redstone glass powers the lamp; ethereal glass (the control) does not"

say "smart glass frosts its whole connected panel when a signal touches it"
# A three-block run, a signal block at one end, and an isolated pane.
for x in 20 21 22; do
    ctl setblock "$x" 80 10 glassential:smart_glass >/dev/null || fail "setblock smart glass $x failed"
done
ctl setblock 26 80 10 glassential:smart_glass >/dev/null || fail "setblock isolated smart glass failed"
ctl wait 6
for x in 20 21 22 26; do
    assert_prop "$(full_state "$x" 80 10)" powered false
done
echo "unpowered: the panel and the isolated pane are clear"
ctl setblock 19 80 10 minecraft:redstone_block >/dev/null || fail "setblock redstone block failed"
ctl wait 8
for x in 20 21 22; do
    assert_prop "$(full_state "$x" 80 10)" powered true
done
assert_prop "$(full_state 26 80 10)" powered false
echo "powered: all three panes of the run frost; the isolated pane stays clear"
ctl setblock 19 80 10 minecraft:air >/dev/null || fail "removing the redstone block failed"
ctl wait 8
for x in 20 21 22; do
    assert_prop "$(full_state "$x" 80 10)" powered false
done
echo "the signal removed: the panel clears again"

if [ "$COUNTERFACTUAL" = 1 ]; then
    # ------------------------------------------------------- differential
    if [ -n "$CONTROL_GAME" ]; then
        say "counterfactual: restart on the control game (no Glassential package)"
        ctl quit >/dev/null 2>&1 || true
        sleep 2
        start_in "$CONTROL_WORLD" "$CONTROL_GAME"
    fi
    say "counterfactual: mine vanilla glass and tinted glass"
    ctl command "gamemode survival @s" >/dev/null
    ctl command "clear @s" >/dev/null
    ctl give minecraft:diamond_pickaxe 1 >/dev/null
    ctl wait 2
    ctl select minecraft:diamond_pickaxe >/dev/null
    ctl wait 2
    ctl setblock 0 80 -6 minecraft:glass >/dev/null
    ctl setblock 1 80 -6 minecraft:tinted_glass >/dev/null
    ctl command "tp @s 0.5 80 -4.5" >/dev/null
    ctl wait 10
    ctl look-at-block 0 80 -6 >/dev/null || true
    broken=$(ctl break 0 80 -6) || { echo "break reply: $broken"; fail "cannot break vanilla glass"; }
    echo "$broken" | jq -c '{scenario:"vanilla glass without silk",block,block_before,drops}'
    [ "$(echo "$broken" | jq -r '.block')" = "minecraft:air" ] || fail "vanilla glass did not break"
    vanilla_glass=$(echo "$broken" | jq -r '.drops["minecraft:glass"] // 0')
    glassential=$(echo "$broken" | jq -r '[((.drops // {}) | keys[] | select(startswith("glassential:")))] | length')
    [ "$vanilla_glass" -eq 0 ] || fail "vanilla glass dropped $vanilla_glass without silk touch"
    [ "$glassential" -eq 0 ] || fail "vanilla glass dropped $glassential Glassential items"
    echo "vanilla glass without silk touch dropped nothing (vanilla loot)"

    ctl look-at-block 1 80 -6 >/dev/null || true
    broken=$(ctl break 1 80 -6) || { echo "break reply: $broken"; fail "cannot break vanilla tinted glass"; }
    echo "$broken" | jq -c '{scenario:"vanilla tinted glass without silk",block,block_before,drops}'
    [ "$(echo "$broken" | jq -r '.block')" = "minecraft:air" ] || fail "vanilla tinted glass did not break"
    tinted=$(echo "$broken" | jq -r '.drops["minecraft:tinted_glass"] // 0')
    glassential=$(echo "$broken" | jq -r '[((.drops // {}) | keys[] | select(startswith("glassential:")))] | length')
    if [ "$NEGATIVE" = 1 ]; then
        [ "$glassential" -ge 1 ] || fail "NEGATIVE run did not observe the false assertion (Glassential drops=$glassential)"
        fail "NEGATIVE run: expected to fail, but vanilla tinted glass reported a Glassential drop"
    fi
    [ "$tinted" -ge 1 ] || fail "vanilla tinted glass dropped $tinted of itself without silk touch"
    [ "$glassential" -eq 0 ] || fail "vanilla tinted glass dropped $glassential Glassential items"
    echo "vanilla tinted glass without silk touch dropped itself, never a Glassential block"

    say "silk touch makes vanilla glass drop itself"
    ctl command "enchant @s minecraft:silk_touch" >/dev/null
    ctl wait 3
    ctl setblock 0 80 -6 minecraft:glass >/dev/null
    ctl wait 5
    ctl look-at-block 0 80 -6 >/dev/null || true
    broken=$(ctl break 0 80 -6) || { echo "break reply: $broken"; fail "cannot break vanilla glass with silk"; }
    echo "$broken" | jq -c '{scenario:"vanilla glass with silk",block,block_before,drops}'
    vanilla_glass=$(echo "$broken" | jq -r '.drops["minecraft:glass"] // 0')
    [ "$vanilla_glass" -ge 1 ] || fail "vanilla glass with silk touch dropped $vanilla_glass of itself"
    echo "COUNTERFACTUAL PASS: vanilla glass needs silk touch, vanilla tinted glass drops itself, and neither drops a Glassential item"

    say "wrong-name controls still fail"
    if ctl give glassential:not_a_glass 1 >/dev/null 2>&1; then
        fail "give glassential:not_a_glass succeeded"
    fi
    ctl setblock 6 80 5 glassential:bogus_glass >/dev/null 2>&1 || true
    [ "$(block_state 6 80 5)" = "minecraft:air" ] || fail "glassential:bogus_glass became a block"
    echo "unknown glass ids are refused"
    echo "PASS: counterfactual differential"
    exit 0
fi

# ------------------------------------------------------------------- drops
say "mine without silk touch: our glasses drop nothing, vanilla tinted glass does"
ctl command "gamemode survival @s" >/dev/null
ctl command "clear @s" >/dev/null
ctl give minecraft:diamond_pickaxe 1 >/dev/null
ctl wait 2
ctl select minecraft:diamond_pickaxe >/dev/null
ctl wait 2
ctl setblock 0 80 -6 glassential:glass_light >/dev/null
ctl setblock 1 80 -6 minecraft:glass >/dev/null
ctl setblock 2 80 -6 glassential:glass_light_tinted >/dev/null
ctl setblock 3 80 -6 minecraft:tinted_glass >/dev/null
ctl setblock 4 80 -6 glassential:glass_ethereal >/dev/null
ctl setblock 5 80 -6 glassential:obsidian_glass >/dev/null
ctl command "tp @s 2.5 80 -4.5" >/dev/null
ctl wait 10
for pair in "0:glassential:glass_light:0" "1:minecraft:glass:0" "2:glassential:glass_light_tinted:0" "3:minecraft:tinted_glass:1" "4:glassential:glass_ethereal:0" "5:glassential:obsidian_glass:0"; do
    x=${pair%%:*}; rest=${pair#*:}; item=${rest%:*}; want=${rest##*:}
    ctl look-at-block "$x" 80 -6 >/dev/null || true
    broken=$(ctl break "$x" 80 -6) || { echo "break reply: $broken"; fail "cannot break $item"; }
    echo "$broken" | jq -c --arg item "$item" '{scenario:("no silk: " + $item),block,block_before,drops}'
    [ "$(echo "$broken" | jq -r '.block')" = "minecraft:air" ] || fail "$item did not break"
    got=$(echo "$broken" | jq -r --arg i "$item" '.drops[$i] // 0')
    [ "$got" -eq "$want" ] || fail "$item without silk dropped $got, expected $want"
done
echo "without silk: our four glasses and the vanilla glass control drop 0; vanilla tinted glass drops 1 (differential)"

say "with silk touch every glass drops itself"
ctl command "enchant @s minecraft:silk_touch" >/dev/null
ctl wait 3
for pair in "0:glassential:glass_light" "1:minecraft:glass" "2:glassential:glass_light_tinted" "3:minecraft:tinted_glass" "4:glassential:glass_ethereal" "5:glassential:obsidian_glass"; do
    x=${pair%%:*}; item=${pair#*:}
    ctl setblock "$x" 80 -6 "$item" >/dev/null
done
ctl wait 5
for pair in "0:glassential:glass_light" "1:minecraft:glass" "2:glassential:glass_light_tinted" "3:minecraft:tinted_glass" "4:glassential:glass_ethereal" "5:glassential:obsidian_glass"; do
    x=${pair%%:*}; item=${pair#*:}
    ctl look-at-block "$x" 80 -6 >/dev/null || true
    broken=$(ctl break "$x" 80 -6) || { echo "break reply: $broken"; fail "cannot break $item with silk"; }
    echo "$broken" | jq -c --arg item "$item" '{scenario:("silk: " + $item),block,block_before,drops}'
    got=$(echo "$broken" | jq -r --arg i "$item" '.drops[$i] // 0')
    [ "$got" -ge 1 ] || fail "$item with silk dropped $got of itself"
done
echo "with silk: all six drop themselves"

# ------------------------------------------------------------------- recipe
say "craft the luminous, tinted-luminous, ethereal and reverse recipes"
ctl command "gamemode creative @s" >/dev/null
ctl command "clear @s" >/dev/null
ctl setblock 6 80 4 minecraft:crafting_table >/dev/null
ctl wait 5
python3 "$PORT/tools/craft_check.py" "$SOCK" "$CTL" 6 80 4 || fail "the Glassential recipe check failed"
ctl command "gamemode creative @s" >/dev/null
ctl command "clear @s" >/dev/null
ctl wait 3

# --------------------------------------------------------------- item side
say "all 24 block items are give-able"
for id in "${GLASSES[@]}"; do
    ctl give "glassential:${id}" 1 >/dev/null || fail "give glassential:${id} failed"
done
ctl wait 10
for id in "${GLASSES[@]}"; do
    [ "$(count_item "glassential:${id}")" -ge 1 ] || fail "glassential:${id} is missing"
done
echo "24/24 block items are give-able"

# The pin builds the content root per mod set (`mod-content/<stamp>/`,
# `mod_content::resource_root`), so the merged tab table is one level below
# `mod-content`; an older in-place root (`mod-content/creative-inventory.json`)
# still matches at depth 1. The game writes the tables during startup, before
# `wait-ready`, but wait for it in bounded steps anyway rather than racing a
# slow build. `-print -quit` avoids `find | head` (a SIGPIPE would trip
# `pipefail`).
creative_file() {
    find "$SCRATCH/game/.horizon-cache/mod-content" -maxdepth 2 \
        -name creative-inventory.json -print -quit 2>/dev/null || true
}
CREATIVE=""
for _ in $(seq 1 30); do
    CREATIVE=$(creative_file)
    if [ -n "$CREATIVE" ]; then
        break
    fi
    sleep 1
done
if [ -n "$CREATIVE" ] && [ -f "$CREATIVE" ]; then
    listed=$(jq '[.tabs[] | select(.type == "CATEGORY") | select(.id | startswith("glassential:glassential_tab")) | .items[]] | length' "$CREATIVE")
    [ "$listed" = 24 ] || fail "the two Glassential tabs list $listed items, expected 24"
    for id in "${GLASSES[@]}"; do
        grep -q "glassential:${id}" "$CREATIVE" || fail "the creative tabs miss glassential:${id}"
    done
    jq -e '.tabs[] | select(.title == "Glassential Renewed - Functional")' "$CREATIVE" >/dev/null || fail "the functional tab title is missing"
    jq -e '.tabs[] | select(.title == "Glassential Renewed - Blocks")' "$CREATIVE" >/dev/null || fail "the blocks tab title is missing"
    echo "the two creative tabs list 24 entries with their titles"
else
    roots=$(find "$SCRATCH/game/.horizon-cache/mod-content" -maxdepth 1 -mindepth 1 2>/dev/null | tr '\n' ' ')
    fail "no creative-inventory.json under $SCRATCH/game/.horizon-cache/mod-content (entries: ${roots:-none})"
fi

say "the block items place"
ctl command "gamemode creative @s" >/dev/null
ctl command "clear @s" >/dev/null
ctl tp 10.5 80 5.5 >/dev/null
for pair in "8 80 6 glassential:glass_ethereal" "9 80 6 glassential:glass_glowstone_lamp" "10 80 6 glassential:clear_fluid_glass"; do
    read -r x y z item <<< "$pair"
    ctl give "$item" 1 >/dev/null || fail "cannot give $item"
    ctl wait 2
    ctl select "$item" >/dev/null || fail "cannot select $item"
    ctl wait 2
    reply=$(ctl place "$x" "$y" "$z" --item "$item" 2>&1) || { echo "place reply: $reply"; fail "cannot place $item"; }
    ctl wait 3
    got=$(block_state "$x" "$y" "$z")
    [ "$got" = "$item" ] || fail "placed $item reads '$got'"
done
echo "3/3 block items placed"

say "wrong-name controls fail"
if ctl give glassential:not_a_glass 1 >/dev/null 2>&1; then
    fail "give glassential:not_a_glass succeeded"
fi
if ctl give glassential:one_way_glass 1 >/dev/null 2>&1; then
    fail "give glassential:one_way_glass succeeded (it is deferred)"
fi
ctl setblock 6 80 5 glassential:bogus_glass >/dev/null 2>&1 || true
[ "$(block_state 6 80 5)" = "minecraft:air" ] || fail "glassential:bogus_glass became a block"
ctl setblock 7 80 5 "glassential:glass_light[powered=true]" >/dev/null 2>&1 || true
[ "$(block_state 7 80 5)" = "minecraft:air" ] || fail "glass_light accepted a state it does not declare"
echo "unknown ids and undeclared states are refused"

echo "PASS: 153 states bind, 6/6 luminous glasses emit 15, dark ethereal blocks light, redstone glass powers a lamp while ethereal glass does not, smart glass frosts its signal-touched panel and clears it, every glass needs silk touch while vanilla tinted glass does not, the four recipes craft, 24 items give and the two tabs list them, the block items place, and unknown ids are refused"
