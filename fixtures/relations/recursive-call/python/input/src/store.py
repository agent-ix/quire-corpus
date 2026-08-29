class Store:
    def recurse(self, n: int) -> int:
        return 0 if n == 0 else self.recurse(n - 1)


def plain_recursion(n: int) -> int:
    return 0 if n == 0 else plain_recursion(n - 1)
