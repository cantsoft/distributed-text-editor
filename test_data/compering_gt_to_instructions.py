import json
import difflib

def reconstruct_text_relative(operations):

    
    inserts = {op["id"]: op for op in operations if op["type_of_operation"] == "i"}

    # --- 2. LOAD DELETES AND REMOVE INSERTS ---
    for op in operations:
        if op["type_of_operation"] == "d":
            before = op["relative_position"][0]
            # delete operations reference the ID of the insert they remove via relative_position[0]
            if before in inserts:
                del inserts[before]     # removes the insert whose ID == before

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

            # Case 1: BOF -> EOF
            if before == -1 and after is None:
                if not order:
                    order.append(op_id)
                    placed.append(op_id)
                

            # Insert at BOF
            elif before == -1:
                if after in order:
                    order.insert(order.index(after), op_id)
                    placed.append(op_id)
                

            # Insert at EOF
            elif after is None:
                if before in order:
                    order.insert(order.index(before) + 1, op_id)
                    placed.append(op_id)
                

            # Insert between before and after
            elif before in order and after in order:
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
    ]   

    print("Testing data sets...")
    for i, (path, name) in enumerate(enumerate_test_datasets):
        print("\n")
        FILENAME = f"{path}_{name}.json"
        
        with open(FILENAME, "r", encoding="utf-8") as f:
            operations = json.load(f)

        reconstructed = reconstruct_text_relative(operations)

        print(f"Reconstructed text from {FILENAME}:")  
        print(reconstructed)

       
        with open(f"{FILENAME}_ground_truth.txt", "r", encoding="utf-8") as f:
            ground_truth = f.read()

        
        diff_lines = difflib.unified_diff(
            ground_truth.splitlines(),
            reconstructed.splitlines(),
            fromfile="ground_truth",
            tofile="reconstructed",
            lineterm=""
        )


        # Consume one entry to check for differences; generator now starts from the second diff line
        if next(diff_lines, None) is not None:
            print("❌ Differences detected between reconstructed text and ground truth.")
            with open(f"debug_out/diff_{name}.txt", "w", encoding="utf-8") as out:
                for line in diff_lines:
                    out.write(line + "\n")
            print(f"Done. Differences saved to diff_{name}.txt in the debug_out folder.")
            print(f"Dataset {i}: {name} is incorrectly designed.")
            flag = False
        else: 
            print("Reconstructed text matches the ground truth.")
            print(f"Dataset {i}: {name} is correctly designed.")
    
    print("\nAll test datasets have been evaluated.")

    if flag:    
        print("All datasets are correctly designed.")
    else:
        print("Some datasets are incorrectly designed.")    
    
