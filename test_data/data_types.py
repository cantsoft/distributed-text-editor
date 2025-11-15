


class letter:
    # CRDT-like wrapper describing a single operation with relative neighbors and metadata
    def __init__(self, char, operacion_before : int, operacion_after : int , letter_id, user_id=None, timestamp=None, type_of_operation="i"):
        self.char = char
        self.relative_position = (operacion_before, operacion_after) 
        """operacion_before: operation id before the character in the document
            operacion_after: operation id after the character in the document  
            this position is relative to other letters positions and it is set as id operation before and after this one   
            
            if pos_before == -1 it means BOF (beginning of file)
            if pos_after == None it means EOF (end of file)
        """
        
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
            'relative_position': self.relative_position,
            'id': self.id,
            'user_id': self.user_id,
            'timestamp': self.timestamp,
            'type_of_operation': self.type_of_operation
        }
