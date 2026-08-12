# Art TODO review

The planned bitmap set in `assets/image_prompts.json` is now complete for the
currently authored 16-passenger roster and the planned UI/gameplay art.

| TODO item | Result |
| --- | --- |
| Menu hero art | Already shipped as `assets/ui/title_background.png`; retained because it matches the current production style. |
| Logo | Added `assets/ui/ui_logo.png`. |
| Driving backgrounds | Added `assets/ui/bg_driving_city.png`, `bg_driving_forest.png`, and `bg_driving_industrial.png`. |
| HUD icons | Added 32×32 alpha PNGs: `icon_fuel.png`, `icon_money.png`, and `icon_time.png`. |
| Item icons | Added 32×32 alpha PNGs: `assets/items/item_generic.png` and `item_locket.png`. |
| Passenger portraits | Existing 16 portraits remain the active roster and were not replaced. |

## Style decision

The original prompt file described pixel art, but the shipped title/cockpit
backgrounds and passenger portraits use cinematic painted noir horror. The new
assets follow the shipped style: near-black teal/green shadows, sodium amber
lights, restrained red supernatural accents, wet surfaces, and distressed
graphic-novel rendering. UI-facing assets keep generous negative space and
strong silhouettes for readability at small sizes.

## Still-open art-adjacent TODOs

The remaining Art section entries are implementation/content tasks rather than
missing files in the planned bitmap set: portrait correspondence and animation,
event-tied feedback, dynamic lighting/headlight effects, and wiring drawn icons
into every existing HUD location. The existing code-native icon atlas already
handles the emoji replacement requirement safely across bitmap-font platforms.
