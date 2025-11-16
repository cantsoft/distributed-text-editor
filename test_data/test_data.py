from dataclasses import dataclass
from typing import List, Tuple, Dict
from random import random, seed, randrange, Random
import sys

seed = randrange(sys.maxsize)
rng = Random(seed)
print("seed:", seed)

peer_id = 123
test_str = "abcdefghijklmnoprstuxyz"


@dataclass
class Operation:
    peer_id: int
    timestapm: int


@dataclass
class Insert(Operation):
    left_op: int | None
    right_op: int | None
    char: str


@dataclass
class Remove(Operation):
    to_remove_op: int


class Node:
    def __init__(self, data: str):
        self.data = data
        self.next: Node | None = None


def to_string(head: Node | None):
    if not (node := head):
        return ""
    s = node.data
    while node := node.next:
        s += node.data
    return s


def get[T](list: List[T], idx: int) -> T | None:
    try:
        return list[idx]
    except:
        return None


def get_serounding_ops(final_pos: int, doc_state: List[Tuple[int, int | None]]) -> Tuple[int | None, int | None, int | None]:
    before_pos = None
    left_op, right_op = None, None
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

    def gen_remove(to_remove_num: int):
        rand = 1 if to_remove_num == 1 else randrange(1, to_remove_num)
        to_remove_num -= 1
        for (idx, (num, fpos)) in enumerate(doc_state):
            if fpos is None:
                rand -= 1
            if rand == 0:
                ops.append(Remove(peer_id, op_num, num))
                doc_state.pop(idx)
                break
        return to_remove_num

    while data_pool:
        op_num += 1
        if random() < 0.5:
            final_pos, ch = data_pool.pop()
            before_pos, left_op, right_op = get_serounding_ops(
                final_pos, doc_state)
            ops.append(Insert(peer_id, op_num, left_op, right_op, ch))
            idx = 0 if before_pos is None else 1+before_pos
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
                    right_op = None if get(
                        doc_state, 0) is None else doc_state[0][0]
                case _ if rand == len(doc_state):
                    left_op = None if get(
                        doc_state, -1) is None else doc_state[-1][0]
                    right_op = None
                case _:
                    left_op, _ = doc_state[rand - 1]
                    right_op, _ = doc_state[rand]
            ops.append(Insert(peer_id, op_num, left_op, right_op, "#"))
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


if __name__ == "__main__":
    try:
        ops = generate_operations(test_str)
        for op in ops:
            print(op)
        doc_str = eval_ops(ops)
        print(doc_str)
    except:
        print("test failed")
    else:
        assert doc_str == test_str

    for i in range(100_000):
        try:
            ops = generate_operations(test_str)
            doc_str = eval_ops(ops)
        except:
            print("test failed")
        else:
            assert doc_str == test_str
