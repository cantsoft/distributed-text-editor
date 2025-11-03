
class letter:
    def __init__(self, char, pos, letter_id, user_id=None, timestamp=None, type_of_operation="i"):
        self.char = char
        self.position = pos 
        """ position in the document relative to other instrucons this means 
        id of letter that is before this one if its adding or id of letter that is to be deleted if its deleting """
        self.id = letter_id # id of this operacion
        self.user_id = user_id # which user made the change
        self.timestamp = timestamp # time of creation
        self.type_of_operation = type_of_operation  # "i" for 'insert' or "d" for 'delete'
    
    def __repr__(self):
        return f"letter(char={self.char}, position={self.position}, site_id={self.site_id})"
    def __str__(self):
        return self.char
    
    def to_json(self):
        return {
            'char': self.char,
            'position': self.position,
            'id': self.id,
            'user_id': self.user_id,
            'timestamp': self.timestamp,
            'type_of_operation': self.type_of_operation
        }