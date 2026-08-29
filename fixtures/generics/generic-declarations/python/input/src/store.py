from typing import Generic, TypeVar

T = TypeVar("T")


class Persist:
    def save(self) -> None:
        pass


class Runner(Generic[T]):
    def run(self) -> None:
        pass


class Store(Persist):
    def save(self) -> None:
        pass
