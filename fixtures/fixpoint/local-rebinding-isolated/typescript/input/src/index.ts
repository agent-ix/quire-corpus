export class Store {
  upsert(): void {}
}

export class Cache {
  upsert(): void {}
}

export function narrow(handle: Store): void {
  handle.upsert();
}

export function other(handle: Cache): void {
  handle.upsert();
}
