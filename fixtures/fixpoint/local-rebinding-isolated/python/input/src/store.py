class Store:
    def upsert(self) -> None:
        pass


class Cache:
    def upsert(self) -> None:
        pass


def narrow(handle: Store) -> None:
    handle.upsert()


def other(handle: Cache) -> None:
    handle.upsert()
