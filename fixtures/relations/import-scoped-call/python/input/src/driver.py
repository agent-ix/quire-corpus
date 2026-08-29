from .store import Store


def drive(handle):
    handle.upsert()
