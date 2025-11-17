import json
from data_types import letter
from wikipedia_data import  get_pages
import random

NUMBER_OF_CHARACTERS = 10
NUMBER_OF_ARTICLES = 10

def create_test_dataset(filename_: str, num_char: int = 1000, type_of_problem: str = "adding" ) -> None:
    """this function creates a test dataset from wikipedia article and saves it to a json file
    
    param filename: 
        filename - name of the file where dataset will be saved
        num_char - number of characters to extract from the article
        type_of_problem - type of problem that dataset will represent :
            "adding" - dataset with adding characters
            "deleting" - dataset with deleting characters
            "mixed" - dataset with both adding and deleting characters
            "timestamps" - dataset that test for removing or adding characters in the same timestamp
            
            
    return: None"""
    
    filename = filename_

    pages = get_pages(num_of_articles=NUMBER_OF_ARTICLES)
    text = ""
    
    for page in pages:
        text += page.text
        if len(text) >= num_char:
            break

    
    letters = []
    wright_text = ""


    if type_of_problem == "adding":
        user_id = 1
        timestamp = 1
        for i in range(min(num_char, len(text))):
            if i == 0:
                letters.append(letter(char=text[i], operacion_before=-1, operacion_after=None, letter_id=i, user_id=user_id, timestamp=timestamp, type_of_operation="i"))
            else:
                letters.append(letter(char=text[i], operacion_before=i-1, operacion_after=None, letter_id=i, user_id=user_id, timestamp=timestamp, type_of_operation="i"))
            
            timestamp += 1                
        filename = filename +"_"+ type_of_problem +".json"
        wright_text = text[:num_char]
        
        
        
        
    elif type_of_problem == "adding_random":
        user_id = 1
        timestamp = 1
        cashe_for_letters = []
        letter_id = 0
        pos_letter = 0  # track the visible position to attach sequential inserts to
        for i in range(min(num_char, len(text))):
            if i == 0:
                letters.append(letter(char=text[i],operacion_before=-1, operacion_after=None, letter_id=letter_id, user_id=user_id, timestamp=timestamp, type_of_operation="i"))
            elif random.random() < 0.5:
                # Cache the letter so it can be inserted later between already placed neighbors
                cashe_for_letters.append((text[i],letter_id-1,letter_id))
                continue
            else:
                letters.append(letter(char=text[i], operacion_before=pos_letter-1, operacion_after=None, letter_id=letter_id, user_id=user_id, timestamp=timestamp, type_of_operation="i"))
            
            
            letter_id += 1
            timestamp += 1  
            pos_letter += 1
                
        last_cashe_id = None
        for i in range(len(cashe_for_letters)):
            # check if the previous letter in cashe has the same operacion_before
            # if so, use last_cashe_id as operacion_before to link them together
            if i>=1 and cashe_for_letters[i-1][1] == cashe_for_letters[i][1] :
                letters.append(letter(char=cashe_for_letters[i][0],operacion_before=last_cashe_id, 
                                    operacion_after=cashe_for_letters[i][2],
                                  letter_id=letter_id, user_id=user_id, timestamp=timestamp, type_of_operation="i"))
            else:
                last_cashe_id = letter_id
                # Flush deferred insert so it lands between its saved before/after neighbors
                letters.append(letter(char=cashe_for_letters[i][0],operacion_before=cashe_for_letters[i][1],
                                       operacion_after=cashe_for_letters[i][2],
                                  letter_id=letter_id, user_id=user_id, timestamp=timestamp, type_of_operation="i"))
            
            letter_id += 1
            timestamp += 1  
            pos_letter += 1
            
            

        filename = filename +"_"+ type_of_problem +".json"
        wright_text = text[:num_char]
        
    elif type_of_problem == "deleting_from_beginning":
        # first add all characters
        user_id = 1
        timestamp = 1
        letter_id = 0
        pos_letter = 0  # ID of the insert that will be removed next when deleting from the head
        for i in range(min(num_char, len(text))):
            if i == 0:
                letters.append(letter(char=text[i],operacion_before=-1, operacion_after=None, letter_id=letter_id, user_id=user_id, timestamp=timestamp, type_of_operation="i"))
            else:
                letters.append(letter(char=text[i], operacion_before=pos_letter-1, operacion_after=None, letter_id=letter_id, user_id=user_id, timestamp=timestamp, type_of_operation="i"))
            
            letter_id += 1
            timestamp += 1  
            pos_letter += 1
        
        # then delete all characters from the head sequentially
        user_id = 1
        pos_letter = 0
        for i in range(min(num_char, len(text))):
            if i == 0:
                letters.append(letter(char=text[i], operacion_before=0, operacion_after=None, letter_id=letter_id, user_id=user_id, timestamp=timestamp, type_of_operation="d"))
            else:
                letters.append(letter(char=text[i], operacion_before=pos_letter, operacion_after=None, letter_id=letter_id, user_id=user_id, timestamp=timestamp, type_of_operation="d"))
            
            letter_id += 1
            timestamp += 1  
            pos_letter += 1 
            
                          
        filename = filename +"_"+ type_of_problem +".json"
        wright_text = ""
        
    elif type_of_problem == "deleting_from_end":
        # first add all characters
        user_id = 1
        timestamp = 1
        letter_id = 0
        pos_letter = 0
        for i in range(min(num_char, len(text))):
            if i == 0:
                letters.append(letter(char=text[i],operacion_before=-1, operacion_after=None, letter_id=letter_id, user_id=user_id, timestamp=timestamp, type_of_operation="i"))
            else:
                letters.append(letter(char=text[i], operacion_before=pos_letter-1, operacion_after=None, letter_id=letter_id, user_id=user_id, timestamp=timestamp, type_of_operation="i"))
            
            letter_id += 1
            timestamp += 1  
            pos_letter += 1
        
        # then delete all characters from the end sequentially
        user_id = 1

        # pos_letter currently equals len(text); we traverse backwards to delete tail-to-head
        for i in range(min(num_char, len(text)) - 1, -1, -1):
            if i == 0:
                letters.append(letter(char=text[i], operacion_before=0, operacion_after=None, letter_id=letter_id, user_id=user_id, timestamp=timestamp, type_of_operation="d"))
            else:
                letters.append(letter(char=text[i], operacion_before=pos_letter-1, operacion_after=None, letter_id=letter_id, user_id=user_id, timestamp=timestamp, type_of_operation="d"))
            
            letter_id += 1
            timestamp += 1  
            pos_letter -= 1     # walk backwards through relative positions to mimic tail deletions
            
            
                         
        filename = filename +"_"+ type_of_problem +".json"
        wright_text = ""
        

    with open(filename, "w", encoding="utf-8") as f:
        json.dump([letter.to_json() for letter in letters], f, ensure_ascii=False, indent=4)

    print(f"Saved {len(letters)} letters to {filename}")

    gt_filename = filename + "_ground_truth.txt"
    
    with open(gt_filename, "w", encoding="utf-8") as file:
        file.write(wright_text)

    print(f"Ground-truth text saved to {gt_filename}")


enumerate_test_datasets = [
    ("data/test_dataset", "adding"),
    ("data/test_dataset", "adding_random"),
    ("data/test_dataset", "deleting_from_beginning"),
    ("data/test_dataset", "deleting_from_end"),
]   


if __name__ == "__main__":
    print("Creating test datasets...")
    for i, (path, name) in enumerate(enumerate_test_datasets):
        print("\n")
        create_test_dataset(path, num_char=NUMBER_OF_CHARACTERS, type_of_problem=name)
        
        print(f"{i}: {name} -> {path}")
    
    print("\nAll test datasets have been created.")