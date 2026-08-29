class Persist:
    def save(self) -> None:
        pass


class Cacheable:
    def evict(self) -> None:
        pass


class Store(Persist, Cacheable):
    pass
