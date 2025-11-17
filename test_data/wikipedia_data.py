import wikipediaapi
import requests
import json
from data_types import letter
from dotenv import load_dotenv
import os

load_dotenv()
EMAIL = os.getenv("WIKI_CONTACT_EMAIL")

def get_pages_from_sub_links(page, n: int, wiki_wiki: wikipediaapi.Wikipedia) -> list:
    """this function gets all sublinks from a given page recursively and loads it into wipiediaapi wraper
    
    param page: 
        page - wikipediaapi page object from which sublinks will be extracted
        n -number of pages to extract
    return: list of wikipediaapi page objects"""
    
    
    links = page.links
    pages = []

    for title in sorted(links.keys()):
        page_py = wiki_wiki.page(links[title].title)
        # print("Page - Exists: %s" % page_py.exists()) #debug line
        if page_py.exists():
            if len(pages) >= n:
                break
            pages.append(page_py)
            # pages += get_pages_from_sub_links(page_py)
    
    else:
        pages += get_pages_from_sub_links(pages[-1], n - len(pages))
        
    
    return pages


def get_pages(article_title : str = 'US history', num_of_articles : int = 10) -> list:
    """this function gets wikipedia pages related to given article title"""
    load_dotenv()
    EMAIL = os.getenv("WIKI_CONTACT_EMAIL")
    
    pages = []

    language_code = 'en'

    wiki_wiki = wikipediaapi.Wikipedia(user_agent=f'teamproject ({EMAIL})',
                                    language=language_code,
                                    extract_format=wikipediaapi.ExtractFormat.WIKI)

    search_query = article_title
    number_of_results = 1
    headers = {
    # 'Authorization': 'Bearer YOUR_ACCESS_TOKEN',
    'User-Agent': f'teamproject ({EMAIL})'
    }


    url = f'https://api.wikimedia.org/core/v1/wikipedia/{language_code}/search/page'
    parameters = {'q': search_query, 'limit': number_of_results}
    response = requests.get(url, headers=headers, params=parameters)


        

    page_py = wiki_wiki.page(response.json()["pages"][0]["key"])

    pages.append(page_py)


    pages += get_pages_from_sub_links(page_py, num_of_articles - 1, wiki_wiki)
        
    
    
    return pages

