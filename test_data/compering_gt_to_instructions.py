import json
import difflib

# def reconstruct_string(letters_json):
    
#     inserts = [op for op in letters_json if op["type_of_operation"] == "i"]
#     by_id = {op["id"]: op for op in inserts}

#     order = []               # final ordered list of letter IDs
#     remaining = set(by_id)   # IDs of letters that still need to be placed
#     last_size = -1

#     # Keep trying to place letters until nothing changes
#     while remaining and last_size != len(remaining):
#         last_size = len(remaining)
#         placed = []

#         for op_id in list(remaining):
#             op = by_id[op_id]
#             before, after = op["relative_position"]

#             # 1) ---------------- BOF → EOF (single isolated letter)
#             if before == -1 and after is None:
#                 if not order:
#                     order.append(op_id)
#                     placed.append(op_id)
#                 continue

#             # 2) ---------------- Insertion at BOF
#             if before == -1:
#                 # insert before `after` (only if `after` is already placed)
#                 if after in order:
#                     idx = order.index(after)
#                     order.insert(idx, op_id)
#                     placed.append(op_id)
#                 continue

#             # 3) ---------------- Insertion at EOF
#             if after is None:
#                 # insert after `before` (only if `before` is already placed)
#                 if before in order:
#                     idx = order.index(before)
#                     order.insert(idx + 1, op_id)
#                     placed.append(op_id)
#                 continue

#             # 4) ---------------- Normal insertion: between `before` and `after`
#             if before in order and after in order:
#                 idx_before = order.index(before)
#                 idx_after = order.index(after)

#                 # Only insert if they appear in logical order
#                 if idx_before < idx_after:
#                     order.insert(idx_after, op_id)
#                     placed.append(op_id)

#         # Remove successfully placed letters
#         for op_id in placed:
#             remaining.remove(op_id)

#     # Fallback: if something couldn’t be placed, append it at the end
#     # (useful for debugging)
#     for op_id in remaining:
#         if op_id not in order:
#             order.append(op_id)

#     # Build final string
#     return "".join(by_id[op_id]["char"] for op_id in order)

# def reconstruct_text(operations):
#     """
#     Reconstructs final text from CRDT operations containing both
#     'i' (insert) and 'd' (delete).
#     """

#     # 1) Separate inserts and deletes
#     inserts = {}
#     deletes = set()

#     for op in operations:
#         if op["type_of_operation"] == "i":
#             inserts[op["id"]] = op
#         elif op["type_of_operation"] == "d":
#             deletes.add(op["id"])

#     # 2) Apply deletes — remove inserts that were deleted
#     active_inserts = {op_id: inserts[op_id] for op_id in inserts if op_id not in deletes}

#     # Document completely deleted?
#     if not active_inserts:
#         return ""

#     by_id = active_inserts

#     # 3) Reconstruct linear ordering using relative positions
#     order = []
#     remaining = set(by_id.keys())
#     last_size = -1

#     while remaining and last_size != len(remaining):
#         last_size = len(remaining)
#         placed = []

#         for op_id in list(remaining):
#             op = by_id[op_id]
#             before, after = op["relative_position"]

#             # Case 1: single letter (BOF → EOF)
#             if before == -1 and after is None:
#                 if not order:
#                     order.append(op_id)
#                     placed.append(op_id)
                    
#             # Case 2: insert at BOF
#             elif before == -1:
#                 if after in order:
#                     idx = order.index(after)
#                     order.insert(idx, op_id)
#                     placed.append(op_id)
                

#             # Case 3: insert at EOF
#             elif after is None:
#                 if before in order:
#                     idx = order.index(before)
#                     order.insert(idx + 1, op_id)
#                     placed.append(op_id)
                                

#             # Case 4: insert between before and after
#             elif before in order and after in order:
#                 idx_before = order.index(before)
#                 idx_after = order.index(after)
#                 if idx_before < idx_after:
#                     order.insert(idx_after, op_id)
#                     placed.append(op_id)

#         for op_id in placed:
#             remaining.remove(op_id)

#     # fallback: append unplaced ops at end (helps debugging)
#     for op_id in remaining:
#         if op_id not in order:
#             order.append(op_id)

#     # 4) Build final text
#     return "".join(by_id[op_id]["char"] for op_id in order)

def reconstruct_text_relative(operations):

    # --- 1. LOAD INSERTS ---
    inserts = {op["id"]: op for op in operations if op["type_of_operation"] == "i"}

    # --- 2. LOAD DELETES AND REMOVE INSERTS ---
    for op in operations:
        if op["type_of_operation"] == "d":
            before = op["relative_position"][0]
            if before in inserts:
                del inserts[before]     # delete removes the insert whose ID == before

    # If everything was deleted
    if not inserts:
        return ""

    # --- 3. REBUILD TEXT FROM REMAINING INSERTS (standard ordering) ---
    by_id = inserts
    order = []
    remaining = set(by_id.keys())
    last = -1

    while remaining and last != len(remaining):
        last = len(remaining)
        placed = []

        for op_id in list(remaining):
            op = by_id[op_id]
            before, after = op["relative_position"]

            # Case 1: BOF → EOF
            if before == -1 and after is None:
                if not order:
                    order.append(op_id)
                    placed.append(op_id)
                continue

            # Insert at BOF
            if before == -1:
                if after in order:
                    order.insert(order.index(after), op_id)
                    placed.append(op_id)
                continue

            # Insert at EOF
            if after is None:
                if before in order:
                    order.insert(order.index(before) + 1, op_id)
                    placed.append(op_id)
                continue

            # Insert between before and after
            if before in order and after in order:
                ib = order.index(before)
                ia = order.index(after)
                if ib < ia:
                    order.insert(ia, op_id)
                    placed.append(op_id)

        for op_id in placed:
            remaining.remove(op_id)

    # fallback: append unplaced
    for op_id in remaining:
        order.append(op_id)

    # --- 4. BUILD FINAL STRING ---
    return "".join(by_id[i]["char"] for i in order)

if __name__ == "__main__":
    
    flag = True
    enumerate_test_datasets = [
        ("data/test_dataset", "adding"),
        ("data/test_dataset", "adding_random"),
        ("data/test_dataset", "deleting_from_beginning"),
        ("data/test_dataset", "deleting_from_end"),
            # ("data/test_dataset", "mixed"),
        # ("data/test_dataset", "timestamps"),
    ]   

    print("Testowanie zbiorów danych...")
    for i, (path, name) in enumerate(enumerate_test_datasets):
        print("\n")
        FILENAME = f"{path}_{name}.json"
        # Wczytanie operacji
        with open(FILENAME, "r", encoding="utf-8") as f:
            operations = json.load(f)

        reconstructed = reconstruct_text_relative(operations)

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
            with open(f"debug_out/diff_{name}.txt", "w", encoding="utf-8") as out:
                for line in diff_lines:
                    out.write(line + "\n")
            print(f"✅ Zrobione. Różnice zapisane w pliku diff_{name}.txt w folderze debug_out.")
            print(f"Zbiór danych {i}: {name} jest źle zaprojektowany.❌")
            flag = False
        else: 
            print("✅ Odtworzony tekst jest identyczny z tekstem wzorcowym.")
            print(f"Zbiór danych {i}: {name} jest dobrze zaprojektowany.✅")
    
    print("\nWszystkie testowe zbiory danych zostały przetestowane.")

    if flag:    
        print("Wszystkie zbiory danych są dobrze zaprojektowane.✅")
    else:
        print("Niektóre zbiory danych są źle zaprojektowane.❌")    
    


