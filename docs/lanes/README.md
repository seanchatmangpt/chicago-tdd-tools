# Lane-result artifacts

Lane work artifacts only (e.g. the N06 disassembly census output).

**Boundary law**: court receipts live in `docs/receipts/` — never place
receipt JSONs here. `dossier emit` reads `docs/receipts/` strictly as
CourtReceipt schema; anything in `docs/lanes/` is invisible to it.
