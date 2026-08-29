class Persist:
    def save(self) -> int:
        return 0


class Store(Persist):
    def __init__(self) -> None:
        self.count = 0

    def exported(self, id: int) -> int:
        return id

    def _crate_visible(self) -> None:
        pass

    def __private_helper(self) -> None:
        pass
