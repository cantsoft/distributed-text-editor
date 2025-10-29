import wikipediaapi
import requests
import json
from data_types import letter
from wikipedia_data import  get_pages

NUMBER_OF_CHARACTERS = 1000
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


    if type_of_problem == "adding":
        site_id = 1
        user_id = 1
        timestamp = 1
        for i in range(min(num_char, len(text))):
            letters.append(letter(char=text[i], pos=i, site_id=site_id, user_id=user_id, timestamp=timestamp, type_of_operation="i"))
            timestamp += 1                
        filename = filename + "_adding.json"
    elif type_of_problem == "deleting":
        site_id = 1
        user_id = 1
        timestamp = 1
        text_inverted = text[::-1]  
        for i in range(min(num_char, len(text_inverted))):
            letters.append(letter(char=text_inverted[i], pos=i, site_id=site_id, user_id=user_id, timestamp=timestamp, type_of_operation="d"))
            timestamp += 1
        filename = filename + "_deleting.json"
    elif type_of_problem == "mixed":
        for i in range(min(num_char, len(text))):
            if i % 2 == 0:
                letters.append(letter(char=text[i], pos=i, site_id=site_id, user_id=user_id, timestamp=timestamp, type_of_operation="i"))
            else:
                letters.append(letter(char=text[i], pos=i, site_id=site_id, user_id=user_id, timestamp=timestamp, type_of_operation="d"))
            timestamp += 1

    elif type_of_problem == "timestamps":
        for i in range(min(num_char, len(text))):
            letters.append(letter(char=text[i], pos=i, site_id=site_id, user_id=user_id, timestamp=timestamp, type_of_operation="i"))
            if i % 5 == 0:
                timestamp += 1

    with open(filename, "w", encoding="utf-8") as f:
        json.dump([letter.to_json() for letter in letters], f, ensure_ascii=False, indent=4)

    print(f"Zapisano {len(letters)} liter do pliku {filename} ✅")





if __name__ == "__main__":
    create_test_dataset("test_dataset", num_char=NUMBER_OF_CHARACTERS, type_of_problem="deleting")