export class Store {
  upsert(): void {}
}

export function first(store: Store): void {
  store.upsert();
}

export function second(store: Store): void {
  store.upsert();
  store.upsert();
}
