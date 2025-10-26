import wikipediaapi
import requests
import json


def get_pages_from_sub_links(page, n):
    """this function gets all sublinks from a given page recursively and loads it into wipiediaapi wraper
    
    param page: 
        page - wikipediaapi page object from which sublinks will be extracted
        n -number of pages to extract
    return: list of wikipediaapi page objects"""
    
    
    links = page.links
    pages = []
    i = 0
    flag = True # flag to check if we reached the limit of n pages if n pages is grater than number of sublinks from page
    # we import subliks from another articles until we reach n pages
    
    for title in sorted(links.keys()):
        page_py = wiki_wiki.page(links[title].title)
        # print("Page - Exists: %s" % page_py.exists()) #debug line
        if page_py.exists():
            if i >= n:
                flag = False
                break
            i+=1
            pages.append(page_py)
            # pages += get_pages_from_sub_links(page_py)
    
    if flag:
        pages += get_pages_from_sub_links(pages[-1], n - len(pages))
        
    
    return pages

pages = []

language_code = 'en'

wiki_wiki = wikipediaapi.Wikipedia(user_agent='teamproject (jan.zakroczymski@gmail.com)',
                                   language=language_code,
                                   extract_format=wikipediaapi.ExtractFormat.WIKI)

search_query = 'US history'
number_of_results = 1
headers = {
  # 'Authorization': 'Bearer YOUR_ACCESS_TOKEN',
  'User-Agent': 'teamproject (jan.zakroczymski@gmail.com)'
}

base_url = 'https://api.wikimedia.org/core/v1/wikipedia/'
endpoint = '/search/page'
url = base_url + language_code + endpoint
parameters = {'q': search_query, 'limit': number_of_results}
response = requests.get(url, headers=headers, params=parameters)

# for page in response.json()["pages"]:
#     print("Page ID: %s, Title: %s" % (page["key"], page["title"]))
    

page_py = wiki_wiki.page(response.json()["pages"][0]["key"])

pages.append(page_py)

# print("Page - Exists: %s" % page_py.exists()) #debug line

# print("Page - tet: %s" % page_py.text)

pages += get_pages_from_sub_links(page_py, 10)

for p in pages:
    print("Page title: %s" % p.title)
    

print("Total pages: %d" % len(pages))

    



