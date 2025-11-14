import json
import difflib

def reconstruct_string(letters_json):
    inserts = [op for op in letters_json if op["type_of_operation"] == "i"]
    by_id = {op["id"]: op for op in inserts}

    order = []               # final ordered list of letter IDs
    remaining = set(by_id)   # IDs of letters that still need to be placed
    last_size = -1

    # Keep trying to place letters until nothing changes
    while remaining and last_size != len(remaining):
        last_size = len(remaining)
        placed = []

        for op_id in list(remaining):
            op = by_id[op_id]
            before, after = op["relative_position"]

            # 1) ---------------- BOF → EOF (single isolated letter)
            if before == -1 and after is None:
                if not order:
                    order.append(op_id)
                    placed.append(op_id)
                continue

            # 2) ---------------- Insertion at BOF
            if before == -1:
                # insert before `after` (only if `after` is already placed)
                if after in order:
                    idx = order.index(after)
                    order.insert(idx, op_id)
                    placed.append(op_id)
                continue

            # 3) ---------------- Insertion at EOF
            if after is None:
                # insert after `before` (only if `before` is already placed)
                if before in order:
                    idx = order.index(before)
                    order.insert(idx + 1, op_id)
                    placed.append(op_id)
                continue

            # 4) ---------------- Normal insertion: between `before` and `after`
            if before in order and after in order:
                idx_before = order.index(before)
                idx_after = order.index(after)

                # Only insert if they appear in logical order
                if idx_before < idx_after:
                    order.insert(idx_after, op_id)
                    placed.append(op_id)

        # Remove successfully placed letters
        for op_id in placed:
            remaining.remove(op_id)

    # Fallback: if something couldn’t be placed, append it at the end
    # (useful for debugging)
    for op_id in remaining:
        if op_id not in order:
            order.append(op_id)

    # Build final string
    return "".join(by_id[op_id]["char"] for op_id in order)




FILENAME = "data/test_dataset_adding_random.json"
# Wczytanie operacji
with open(FILENAME, "r", encoding="utf-8") as f:
    operations = json.load(f)

reconstructed = reconstruct_string(operations)

print(f"Odtworzony tekst z pliku {FILENAME}:")  
print(reconstructed)

# Wczytanie tekstu wzorcowego
with open(f"{FILENAME}_ground_truth.txt", "r", encoding="utf-8") as f:
    ground_truth = f.read()

# Porównanie i zapis konfliktów do pliku
diff_lines = difflib.unified_diff(
    ground_truth.splitlines(),
    reconstructed.splitlines(),
    fromfile="ground_truth",
    tofile="reconstructed",
    lineterm=""
)


if next(diff_lines, None) is not None:
    print("❌ Wykryto różnice między odtworzonym tekstem a tekstem wzorcowym.")
    with open("debug_out/diff.txt", "w", encoding="utf-8") as out:
        for line in diff_lines:
            out.write(line + "\n")
    print("✅ Zrobione. Różnice zapisane w pliku diff.txt w folderze debug_out.")
else: 
    print("✅ Odtworzony tekst jest identyczny z tekstem wzorcowym.")


