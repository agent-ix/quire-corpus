from .store import Store


def drive() -> None:
    store: Store = Store()
    store.upsert()
