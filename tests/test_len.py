import unittest
import free_range_rust
from free_range_rust import Space


class TestLen(unittest.TestCase):

    def test_len_Discrete(self)->None:
        space = Space.Discrete(10, start=0)
        self.assertEqual(len(space), 10)

    def test_len_Box(self)->None:
        space = Space.Box(
            low=[0],
            high=[1],
        )
        self.assertEqual(len(space), 1)

        space = Space.Box(
            low=[0, 0],
            high=[1, 1],
        )
        self.assertEqual(len(space), 2)

    def test_len_Dict(self)->None:
        space = Space.Dict({
            'a': Space.Discrete(10, start=0),
            'b': Space.Box(
                low=[0],
                high=[1],
            ),
        })
        self.assertEqual(len(space), 2)

    def test_len_Tuple(self)->None:
        space = Space.Tuple([
            Space.Discrete(10, start=0),
            Space.Box(
                low=[0],
                high=[1],
            ),
        ])
        self.assertEqual(len(space), 2)

    def test_len_OneOf(self)->None:
        space = Space.OneOf([
            Space.Discrete(10, start=0),
            Space.Box(
                low=[0],
                high=[1],
            ),
        ])
        self.assertEqual(len(space), 2)