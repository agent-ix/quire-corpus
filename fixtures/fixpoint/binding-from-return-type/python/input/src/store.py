class Store:
    def upsert(self) -> None:
        pass


def make() -> Store:
    return Store()


def drive() -> None:
    store = make()
    store.upsert()
