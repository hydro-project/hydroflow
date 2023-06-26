import random
import numpy as np


class Node:
    def __init__(self, data):
        self.data = data
        self.left = None
        self.right = None


def insert_random_leaf(root, data):
    if random.choice([True, False]):
        if root.left is None:
            root.left = Node(data)
        else:
            insert_random_leaf(root.left, data)
    else:
        if root.right is None:
            root.right = Node(data)
        else:
            insert_random_leaf(root.right, data)


def create_binary_tree(data_list):
    root = Node(data_list[0])
    for data in data_list[1:]:
        insert_random_leaf(root, data)
    return root


def find_max_length(root):

    if not root:
        return 0
    
    def helper(root):

        if not root:
            return 0

        if root.left and root.right:
            return max(1 + helper(root.left), 1 + helper(root.right))
        if root.left:
            return 1 + helper(root.left)
        if root.right:
            return 1 + helper(root.right)
        if not root.left and not root.right:
            return 0
    
    return 2 + helper(root.left) + helper(root.right)


lengths = []

for _ in range(100):
    data_list = np.arange(10000)
    binary_tree = create_binary_tree(data_list)
    lengths.append(find_max_length(binary_tree))

print(sum(lengths) / len(lengths))
