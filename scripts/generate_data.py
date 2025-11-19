import os
import sys
import json
import pathlib

from dataclasses import dataclass
from typing import List, Tuple, Dict
from random import random, seed, randrange, Random


PEER_ID = 123
TEST_STR = "abcdefghijklmnoprstuxyz"
OUTPUT_DIR: str = os.path.join(pathlib.Path(__file__).parent.resolve(), "../data")
OUTPUT_FILENAME: str = "relative_insert_remove.json"


@dataclass
class Operation:
    peer_id: int
    timestamp: int

    def __dict__(self) -> dict:
        return {
            "peer_id": self.peer_id,
            "timestamp": self.timestamp
        }


@dataclass
class Insert(Operation):
    left_op: int | None
    right_op: int | None
    char: str

    def __dict__(self) -> dict:
        return {
            "op_type": "insert",
            **super(Insert, self).__dict__(),
            "left_op": self.left_op if self.left_op is not None else -1,
            "right_op": self.right_op if self.right_op is not None else -1,
            "char": self.char
        }


@dataclass
class Remove(Operation):
    to_remove_op: int

    def __dict__(self) -> dict:
        return {
            "op_type": "remove",
            **super(Remove, self).__dict__(),
            "to_remove_op": self.to_remove_op
        }


class Node:
    def __init__(self, data: str):
        self.data = data
        self.next: Node | None = None


def to_string(head: Node | None) -> str:
    if not (node := head): return ""
    s = node.data
    while node := node.next: s += node.data
    return s


def get[T](list: List[T], idx: int) -> T | None:
    try: return list[idx]
    except IndexError: return None


def get_serounding_ops(final_pos: int, doc_state: List[Tuple[int, int | None]]) -> Tuple[int | None, int | None, int | None]:
    before_pos, left_op, right_op = None, None, None
    for (pos, (op_num, fpos)) in enumerate(doc_state):
        if fpos is not None and fpos > final_pos:
            right_op = op_num
            break
        before_pos = pos
        left_op = op_num
    return before_pos, left_op, right_op,


def generate_operations(data: str) -> List[Remove | Insert]:
    data_pool = {elem for elem in enumerate(data)}
    doc_state: List[Tuple[int, int | None]] = []
    to_remove_num = 0
    ops: List[Remove | Insert] = []
    op_num = -1

    def gen_remove(to_remove_num: int) -> int:
        rand = 1 if to_remove_num == 1 else randrange(1, to_remove_num)
        to_remove_num -= 1
        for (idx, (num, fpos)) in enumerate(doc_state):
            if fpos is None:
                rand -= 1
            if rand == 0:
                ops.append(Remove(PEER_ID, op_num, num))
                doc_state.pop(idx)
                break
        return to_remove_num

    while data_pool:
        op_num += 1
        if random() < 0.5:
            final_pos, ch = data_pool.pop()
            before_pos, left_op, right_op = get_serounding_ops(final_pos, doc_state)
            ops.append(Insert(PEER_ID, op_num, left_op, right_op, ch))
            idx = 0 if before_pos is None else 1 + before_pos
            doc_state.insert(idx, (op_num, final_pos))
        elif random() < 0.5:
            if to_remove_num == 0:
                op_num -= 1
                continue
            to_remove_num = gen_remove(to_remove_num)
        else:
            to_remove_num += 1
            rand = randrange(1 + len(doc_state))
            match rand:
                case 0:
                    left_op = None
                    right_op = None if get(doc_state, 0) is None else doc_state[0][0]
                case _ if rand == len(doc_state):
                    left_op = None if get(doc_state, -1) is None else doc_state[-1][0]
                    right_op = None
                case _:
                    left_op, _ = doc_state[rand - 1]
                    right_op, _ = doc_state[rand]
            ops.append(Insert(PEER_ID, op_num, left_op, right_op, "#"))
            doc_state.insert(rand, (op_num, None))
    while to_remove_num != 0:
        op_num += 1
        to_remove_num = gen_remove(to_remove_num)
    return ops

def eval_ops(ops: List[Remove | Insert]) -> str:
    doc_list: Node | None = None
    op_to_node: Dict[int, Node] = {}
    for (op_num, op) in enumerate(ops):
        match op:
            case Insert(_, _, left_op, _, ch):
                new = Node(ch)
                op_to_node[op_num] = new
                if left_op is None:
                    new.next = doc_list
                    doc_list = new
                else:
                    before = op_to_node[left_op]
                    if after := before.next:
                        new.next = after
                    before.next = new
            case Remove(_, _, op_num):
                to_remove_node = op_to_node[op_num]
                if doc_list is to_remove_node:
                    doc_list = doc_list.next  # type: ignore
                    continue
                node = doc_list  # type: ignore
                while node.next != to_remove_node:  # type: ignore
                    node = node.next  # type: ignore
                if node.next.next is None:  # type: ignore
                    node.next = None  # type: ignore
                else:
                    node.next = node.next.next  # type: ignore
    return to_string(doc_list)  # type: ignore


def write_file(input_data: List[Insert | Remove]) -> None:
    if not os.path.exists(OUTPUT_DIR): os.mkdir(OUTPUT_DIR)

    output_data: dict = {"data": [data.__dict__() for data in input_data]}
    with open(os.path.join(OUTPUT_DIR, OUTPUT_FILENAME), "w") as output_file:
        json.dump(output_data, output_file)

if __name__ == "__main__":

    try: ops = generate_operations(TEST_STR)
    except IndexError:
        print("test failed")
        sys.exit(-1)

    doc_str = eval_ops(ops)
    assert doc_str == TEST_STR

    write_file(ops)