class Store:
    def upsert(self) -> None:
        pass


class Cache:
    def upsert(self) -> None:
        pass


def drive() -> None:
    a = b
    b = c
    c = a
    a.upsert()
