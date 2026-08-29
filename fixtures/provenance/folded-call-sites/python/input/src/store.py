class Store:
    def upsert(self) -> None:
        pass


def first(store: Store) -> None:
    store.upsert()


def second(store: Store) -> None:
    store.upsert()
    store.upsert()
