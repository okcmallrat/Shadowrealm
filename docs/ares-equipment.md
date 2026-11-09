# Ares firearms, ammunition and inventory selection

The Redmond Ares Weapon Emporium now sells its original firearms and ammunition.
Courier earnings and Michael's hidden credstick can fund a gun, which can be
equipped, fired and reloaded through the existing native combat loop.

From the default Redmond start, visit Ares in the southeast (location 11).
Read the shopkeeper's passage and choose Browse. Weapons lists the original stock;
Equipment includes ammo clips. Up/Down and Enter/Z select an item. Confirmation
defaults to No, as in the original. Choose Yes to buy; Back and Leave close the
menus. Purchases go to the currently selected runner. Select a different runner
with Tab before entering the shop. Courier service takes priority when that shop
is the active run's matching pickup or delivery.

Press I between actions to open that runner's inventory. Choose a firearm and
Enter/Z equips it; choosing the equipped firearm unequips it. Close returns to
play. Buying a gun does not automatically equip it. F fires and existing finite
reserves supply automatic reloads. G remains the older first-inventory-weapon
shortcut. Slots marked with an asterisk are unavailable in this bounded menu.

## Original references and native rules

The user's verified Ghidra reference supplied targeted views at `$1fa78`,
`$56ac0`, `$5794e`, `$57a44`, `$57a6e`, `$57b04`, `$12168`, `$125fa` and `$36fc`.
Private generated exports remain in `/workspace/work/shops/ghidra`; the supplied
ROM and GZF are preserved. Actual instructions and original-routine comparisons
validate the translation. No emulated CPU executes inside the desktop game.

`data/shops/ares.json` is a small typed Serde catalog of verified pointers and
the exact shop/district/location scope. `src/shops.rs` executes the rules in Rust.
Unknown JSON fields and references outside that scope are rejected. Names,
stock, prices, greeting and refusal passages are read from the ROM.

- `$1fa78` selects Redmond location 11, node `$110dd2`, shop 1 and shop type -1.
  Its C-B contact opens Ares after any matching courier service. No other shop
  entrances are enabled by this addition.
- Each merchandise row is three bytes: zero-based item ID, inventory metadata
  and price multiplier. Ares uses its eight-row blocks in `$1aac86` (weapons)
  and `$1aad77` (equipment). Negative item bytes denote empty rows.
- `$57a44` reads the item's base-price word and applies nonzero multipliers in
  tenths. Ammo row `$45` instead uses its third byte directly: 25 nuyen here.
  `$57b04` uses the selected runner's negotiating byte and Ares' threshold at
  `$1aac6e + 2`. Signed-byte comparisons, truncated arithmetic and rounding are
  retained; Ares does not receive another shop's contact discount.
- `$5794e` scans eight inventory slots for merchandise, then checks funds before
  confirmation. Successful purchases fill the first empty slot with ID + 1 and
  copy its metadata. Ammo bypasses the inventory-space check, adds one reserve
  clip and uses the original 20-clip cap. The funds check precedes that cap.
  Byte overflow and the original cap behavior for malformed counts are retained.
  Refused or cancelled purchases do not charge money. No stock depletion is
  invented: the original represented routine keeps the ROM stock unchanged.
- `$12168` explicitly selects a firearm slot. `$125fa` converts the old magazine
  into a percentage, then the new capacity scales it with the original rounding.
  The same-slot branch toggles an actual equipped weapon off; an empty old weapon
  goes directly to equipping, even if its stale slot byte matches the chosen slot.
  Spell/unarmed returns retain percentage resources. Inactive actors do not gain
  ammo through `$36fc`. Old non-firearm contexts with unresolved incoming D1 are
  rejected without mutation.

The host stock/inventory panels replace the original full shop and Pocket
Secretary UI. They freeze the world and runner selection while open. Price quotes,
purchase results, inventory pages and a cloned HUD/scene are prepared before
wallet or actor state commits, including invalid-asset failures.

Supported merchandise is Ares' five firearms and ammo clips. Grenades, patches,
other consumables, armor purchases/equipping, accessories, selling, trading,
other shops, original UI/audio and full inventory actions remain unavailable.
The existing equipment icons/health/resource HUD show the selected gun and ammo;
the I panel and its labels are host presentation.

## Validation

```sh
cargo build --release --locked
python tools/validate_shops.py /path/to/Shadowrun.bin
SHADOWREALM_TEST_ROM=/path/to/Shadowrun.bin cargo test --locked -- --include-ignored
```

The new oracle executes the original Ares location selection, 4,096 complete
purchase decisions and 4,096 complete firearm selections. It compares every actor
byte, nuyen, prices and stock eligibility, covering all stock rows, all negotiating
byte values, empty/full inventory, cancellation, insufficient funds, clip limits,
byte overflow, party slots, toggles, spell returns and inactive actors. Only
display/wait/confirmation input is omitted for purchases; only HUD/menu redraw is
omitted for firearm selection. Gameplay instructions remain intact. Existing
courier and Stoker/taxi original-routine checks also pass.

The connected controller test starts at the normal Redmond spawn with the
development roster and 20 nuyen. Six real courier jobs fund Michael's belongings,
leaving 530 nuyen. Joshua walks to Ares, buys a 500-nuyen Warhawk and a 25-nuyen
clip, equips it through I, then fires seven shots with F. His six-round magazine
empties and one real reload consumes a reserve. The ending balance is 5, clips
go 10 -> 11 on purchase -> 10 on reload, and five rounds remain at tick 15339.
No actor/stat/equipment, money, RNG, position, map or job-result assignments occur
after normal session construction. No encounter spawned; this establishes
shopping/equipment/fire/reload, not a firefight along that route.

Additional explicit fixtures verify full-inventory ammo purchases, cap/refusal
order, world/selection freezing, equipment toggling, unimplemented item guards,
cancellation and failed HUD rollback for both purchases and equipment changes.
Rendered stock, confirmation, inventory and firing HUD panels were inspected.
All 71 unit/pure tests and 98 ROM-backed integration tests pass. Windows GUI/input
playtesting remains pending.

Native Linux desktop and Windows release builds pass. The refreshed Windows ZIP
is 580,651 bytes. CRCs, all 22 payload hashes, README/executable identity, x64 PE
format and EXE/DLL imports are verified. SHA-256:
`4092db2ce63f42c48aaead732629b980ebc8d2ee83a21f5d6c3d9f44d30ba27f`.
The ROM and Ghidra database are excluded. These changes and the package remain
local; this continuation has not been published.
