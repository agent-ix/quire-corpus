from .store import Store


def drive(store: Store) -> None:
    store.upsert()
