
class letter:
    def __init__(self, char, pos, site_id = "en", user_id=None, timestamp=None, type_of_operation="i"):
        self.char = char
        self.position = pos
        self.site_id = site_id
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
            'site_id': self.site_id,
            'user_id': self.user_id,
            'timestamp': self.timestamp,
            'type_of_operation': self.type_of_operation
        }