import json
import difflib

FILENAME = "data/test_dataset_adding_random.json"


def reconstruct_text(ops):
    # ops = sorted(ops, key=lambda x: x["timestamp"])
    text = []
    for op in ops:
        if op["id"] == 0:
            text.append(op["char"])
            continue
        if op["type_of_operation"] == "i":
            text.insert(op["position"] + 1, op["char"])
        elif op["type_of_operation"] == "d":
            if 0 <= op["position"] < len(text):
                text.pop(op["position"])
    return "".join(text)


# Wczytanie operacji
with open(FILENAME, "r", encoding="utf-8") as f:
    operations = json.load(f)

reconstructed = reconstruct_text(operations)

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


