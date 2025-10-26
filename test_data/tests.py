import json

# wczytaj dane z pliku
with open("letters.json", "r", encoding="utf-8") as f:
    letters = json.load(f)

# odtwórz tekst z liter (zakładamy, że w to_json() jest pole 'char' lub podobne)
text = "".join(letter["char"] for letter in letters)

print(text)