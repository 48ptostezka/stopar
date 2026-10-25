# Jak psát lokace

Celá hra je jeden soubor `games/<hra>/game.toml`. Obrázky patří do
`games/<hra>/images/`. Všechny texty jsou česky i anglicky.

Hotové příklady najdeš ve dvou hrách:

- **`games/testing/`** má sedm míst a každé ukazuje jednu věc;
- **`games/galaxy/`** je ukázka skutečného příběhu.
  **Kontrola bez spouštění aplikace:**

```sh
cargo run -p stopa-codes -- games/<hra>/game.toml > codes.html
```

Příkaz vypíše chyby a varování a zároveň vytvoří stránku s QR kódy k tisku.
Stejné kontroly proběhnou i při `trunk build`. Hra s chybou se nesestaví.

---

## 1. Kostra hry

```toml
[game]
id = "galaxy"                 # je v každém QR kódu, po vytištění kódů neměnit
skin = "orbit"                # vzhled: orbit, trail nebo case
default_language = "cs"
title = { cs = "Název hry", en = "Game title" }

[items.lampa]                 # předmět, „lampa“ je jeho id
icon = "lantern"
name = { cs = "Lampa", en = "Lamp" }
description = { cs = "Svítí ve tmě.", en = "It shines in the dark." }

[clues.stopy_v_bahne]         # stopa do deníku
title = { cs = "Stopy v bahně", en = "Tracks in the mud" }
text = { cs = "Vedou k řece.", en = "They lead to the river." }
```

- **Id** (`lampa`, `stopy_v_bahne`, …) piš malými písmeny bez diakritiky,
  případně s číslicemi, `_` a `-`. Hráč je nikdy neuvidí.
- **Ikony předmětů:** `generic`, `map`, `key`, `coin`, `lantern`, `feather`,
  `cell`, `crystal`, `decoder`, `tank`, `letter`, `flask`, nebo vlastní ikona
  ([kap. 11](#11-vlastní-ikony-předmětů)).
- **Pořadí:** inventář ukazuje předměty v pořadí, v jakém jsou v souboru.

---

## 2. První lokace

Lokace je místo s QR kódem. Její příběh se skládá ze **scén** a každá scéna
je jedna obrazovka.

```toml
[locations.mlyn]
code = "K7Q2-MLYN"            # 6–32 velkých písmen, číslic a pomlček
name = { cs = "Starý mlýn", en = "Old mill" }

[locations.mlyn.scenes.vchod] # první scéna v souboru = začátek
text = { cs = "Dveře mlýna jsou pootevřené.", en = "The mill door stands ajar." }
next = "uvnitr"               # tlačítko Pokračovat

[locations.mlyn.scenes.uvnitr]
text = { cs = "Uvnitř je ticho.", en = "It's quiet inside." }
give = { items = { lampa = 1 } }
# žádné next = tady příběh končí a ukáže se Hotovo
```

`code` musí být jedinečný a těžko uhodnutelný. Jakmile kódy vytiskneš, už ho
neměň.

---

## 3. Jak scéna končí

Každá scéna končí **nejvýš jedním** z těchto způsobů:

| V souboru        | Hráč uvidí                                  |
| ---------------- | ------------------------------------------- |
| `next = "scena"` | tlačítko **Pokračovat**                     |
| `[[…choices]]`   | tlačítka s **volbami** ([kap. 7](#7-volby)) |
| `[…password]`    | pole na **heslo** ([kap. 8](#8-heslo))      |
| nic              | tlačítko **Hotovo**, příběh místa tu končí  |

---

## 4. Text

```toml
text = { cs = "Krátký text.", en = "Short text." }
```

Delší text se píše takhle. Prázdný řádek začne nový odstavec:

```toml
text.cs = """
První odstavec.

Druhý odstavec.
"""
text.en = """
First paragraph.

Second paragraph.
"""
```

**Odstavec, který se ukáže jen někdy:**

```toml
[[locations.mlyn.scenes.uvnitr.paragraphs]]
when = { items = { lampa = 1 } }
text = { cs = "Lampa ozáří starý mlýnský kámen.", en = "The lamp lights up an old millstone." }
```

Odstavce se ukazují pod hlavním textem, a to všechny, jejichž podmínka zrovna
platí.

---

## 5. Podmínky

Podmínku (`when` a `requires`) lze složit z těchto částí. Musí platit všechny
najednou.

| Klíč            | Platí, když…                       | Příklad                      |
| --------------- | ---------------------------------- | ---------------------------- |
| `items`         | hráč má aspoň tolik kusů           | `items = { mince = 3 }`      |
| `clues`         | hráč má tyto stopy                 | `clues = ["stopy_v_bahne"]`  |
| `flags`         | jsou nastavené tyto příznaky       | `flags = ["pomohl_mlynari"]` |
| `visited`       | hráč už tato místa naskenoval      | `visited = ["mlyn"]`         |
| `without_items` | hráč tyto předměty **nemá**        | `without_items = ["lampa"]`  |
| `without_clues` | hráč tyto stopy **nemá**           | `without_clues = ["…"]`      |
| `without_flags` | tyto příznaky **nejsou** nastavené | `without_flags = ["…"]`      |

```toml
when = { items = { mince = 2 }, without_flags = ["okraden"] }
```

---

## 6. Odměny a příznaky

| Klíč   | Co dělá             | Kde ho lze použít   |
| ------ | ------------------- | ------------------- |
| `give` | dá předměty a stopy | scéna, volba, heslo |
| `set`  | nastaví příznaky    | scéna, volba, heslo |
| `take` | spotřebuje předměty | jen u volby         |

```toml
give = { items = { mince = 2, lampa = 1 }, clues = ["stopy_v_bahne"] }
set = ["pomohl_mlynari"]
```

**Příznak** je neviditelná poznámka o tom, co hráč udělal, třeba
`vzal_mapu`. Nikde ho není potřeba deklarovat: stačí ho nastavit přes `set`
a později testovat přes `flags` / `without_flags`. Překlep v příznaku
kontrola ohlásí.

**Odměnu dostane hráč vždy jen jednou**, ať se na místo vrátí kolikrát chce.

---

## 7. Volby

```toml
[locations.mlyn.scenes.uvnitr]
text = { cs = "Mlynář ti nabízí obchod.", en = "The miller offers you a deal." }

[[locations.mlyn.scenes.uvnitr.choices]]
id = "koupit"
label = { cs = "Koupit mapu (2 mince)", en = "Buy the map (2 coins)" }
take = { mince = 2 }
locked = { cs = "Mapa stojí 2 mince", en = "The map costs 2 coins" }
give = { items = { mapa = 1 } }
to = "konec"

[[locations.mlyn.scenes.uvnitr.choices]]
id = "odejit"
label = { cs = "Odejít", en = "Leave" }
to = "konec"
```

| Klíč          | Význam                                           |
| ------------- | ------------------------------------------------ |
| `id`          | jméno volby, v rámci scény jedinečné             |
| `label`       | text na tlačítku                                 |
| `when`        | podmínka; když neplatí, volba je skrytá…         |
| `locked`      | …nebo šedá s tímto vysvětlením                   |
| `take`        | spotřebuje předměty; bez nich volba nejde zvolit |
| `give`, `set` | odměny a příznaky                                |
| `to`          | kam volba vede; bez `to` příběh skončí           |

**Rozhodnutí, nebo nabídka?** Rozhoduje o tom `pick` u scény:

- **`pick = "one"`** (výchozí) je **konečné rozhodnutí**. Po volbě ostatní
  možnosti zešednou a už se neotevřou: ani po novém skenování, ani tlačítkem
  Zpět. Zvolenou možnost jde zmáčknout znovu kvůli přečtení, ale nic nového
  nedá.
- **`pick = "any"`** je **nabídka**. Všechny možnosti zůstávají otevřené
  a každá dá odměnu jednou. Hodí se na obchod, rozhovory s postavami nebo
  „rozhlédnout se kolem“. Volba s `to` na tutéž scénu (nebo `next` z konce
  rozhovoru) vrátí hráče zpět do nabídky. Pod nabídkou je vždy tlačítko
  **Hotovo**, kterým hráč místo kdykoli opustí.

---

## 8. Heslo

```toml
[locations.mlyn.scenes.dvere]
text = { cs = "Na dveřích je zámek s písmeny.", en = "The door has a letter lock." }

[locations.mlyn.scenes.dvere.password]
answers = ["KAMEN", "mlýnský kámen"]   # víc správných odpovědí je v pořádku
prompt = { cs = "Heslo", en = "Password" }
hint = { cs = "Co mele obilí?", en = "What grinds the grain?" }
wrong = { cs = "Zámek nepovolí.", en = "The lock won't budge." }
give = { clues = ["otevreno"] }
to = "sklep"
```

Na velikosti písmen, diakritice, mezerách ani interpunkci nezáleží:
„Kámen“, „KAMEN“ i „k-a-m-e-n“ projdou. Počet pokusů není omezený. Povinné je
jen `answers`.

---

## 9. Scéna, která čeká

Když má rozhodnutí volbu, ke které hráč potřebuje předmět, mohl by se
rozhodnout dřív, než ho získá, a o volbu by přišel. Proto nech scénu
**počkat**:

```toml
[locations.mlyn.scenes.sklep]
text = { cs = "Ve sklepě je tma.", en = "The cellar is dark." }
requires = { items = { lampa = 1 } }
waiting = { cs = "Bez lampy nic neuvidíš. Vrať se s ní.", en = "You can't see without a lamp. Come back with one." }
```

Dokud podmínka neplatí, hráč uvidí text scény, hlášku `waiting` a jen
tlačítko Hotovo: žádné volby, heslo ani Pokračovat. Když místo naskenuje
znovu, vrátí se do této scény. Jakmile lampu má, objeví se celé pokračování.

Kontrola tě sama **varuje**, když jde rozhodnutí udělat moc brzy. Pokud je
ztráta volby záměr, třeba odměna pro připravené, napiš ke scéně
`allow_lockout = true`.

---

## 10. Obrázky

```toml
image = "mlyn.svg"            # soubor v games/<hra>/images/
alt = { cs = "Starý mlýn u řeky", en = "An old mill by a river" }
```

Formáty: png, svg, jpg, webp. Obrázek se ukáže nad textem scény a funguje
i bez internetu. Krátký popis `alt` je pro hráče se čtečkou obrazovky.

---

## 11. Vlastní ikony předmětů

Ulož SVG soubor do `games/<hra>/icons/` a použij jeho jméno:

```toml
[items.propustka]
icon = "propustka.svg"
```

Aby ikona ladila s ostatními, kresli ji stejně jako vestavěné:

```xml
<svg xmlns="http://www.w3.org/2000/svg" viewBox="-16 -16 32 32">
  <path d="M-12 -8H12V-3A3 3 0 0 0 12 3V8H-12V3A3 3 0 0 0 -12 -3Z"/>
  <path d="M4 -8V-5M4 -1V1M4 5V8"/>
</svg>
```

- **Mřížka:** 32 × 32 se středem v 0,0 (`viewBox="-16 -16 32 32"`). Při
  stejné mřížce budou čáry stejně silné jako u vestavěných ikon.
- **Jen obrysy, žádné barvy.** Barvu a tloušťku čáry dodá vzhled hry. Kdo
  uvede `fill` nebo `stroke`, má barvu napevno a ta se se vzhledem měnit
  nebude.
- **Ikona je součástí aplikace,** takže funguje i bez internetu.
- **Kontrola ohlásí** chybějící soubor, soubor, který není SVG, i SVG se
  skriptem (`<script>`, `onload=…`). Ohlásí také překlep v názvu vestavěné
  ikony.
  Ukázka je v testovací hře: `games/testing/icons/propustka.svg`.

---

## Co platí vždy

- **Odměny jen jednou.** Scéna, volba i heslo dají svou odměnu jen poprvé.
- **Nové skenování pokračuje tam, kde hráč skončil.** Pokud příběh místa už
  skončil, začne znovu od začátku kvůli přečtení. Rozhodnutí přitom zůstanou
  platit.
- **Na scénu se dá dostat jen hraním.** Ruční adresa ani druhá karta
  prohlížeče pravidla neobejdou.
- **Začátek.** Příběh začíná první scénou v souboru, nebo scénou, kterou
  určíš klíčem `start = "…"` u lokace.

---

## Na co si dát pozor

1. **Pořadí v TOML.** Vše, co patří přímo ke scéně (`text`, `next`, `pick`,
   `requires`, `give`…), piš **před** první `[[…paragraphs]]`,
   `[[…choices]]` nebo `[…password]`. Co stojí za nimi, patří už k nim.

```toml
   [locations.mlyn.scenes.uvnitr]
   text = { … }
   pick = "any"                     # ✔ ještě patří ke scéně

   [[locations.mlyn.scenes.uvnitr.choices]]
   id = "a"
   # pick = "any"                   # ✘ tady by patřilo k volbě a kontrola by ohlásila chybu
```

2. **Po startu hry neměň id.** Hráči mají uložený postup podle id scén,
   voleb, předmětů a stop. `game.id` a `code` lokací jsou navíc ve
   vytištěných QR kódech.
3. **Vždy obě řeči.** Každý text potřebuje `cs` i `en`. Chybějící překlad je
   chyba.
4. **Kódy a hesla nejsou tajné před šikovným hráčem.** Jsou uložené uvnitř
   aplikace.
