class Store:
    def upsert(self) -> None:
        pass


def drive(first: Store) -> None:
    second = first
    third = second
    third.upsert()
