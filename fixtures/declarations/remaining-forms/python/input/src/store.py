import functools


class Outer:
    class Inner:
        def deep(self) -> None:
            pass

    @staticmethod
    def static_helper() -> None:
        pass

    @classmethod
    def class_helper(cls) -> None:
        pass


async def fetch() -> None:
    pass


@functools.cache
def decorated() -> int:
    return 1


def host() -> None:
    def nested() -> None:
        pass
