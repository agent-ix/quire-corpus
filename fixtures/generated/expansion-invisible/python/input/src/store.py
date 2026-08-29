import functools


def wrap(fn):
    @functools.wraps(fn)
    def inner(*args, **kwargs):
        return fn(*args, **kwargs)

    return inner


@wrap
def written_literally() -> None:
    pass


globals()["generated"] = wrap(written_literally)
