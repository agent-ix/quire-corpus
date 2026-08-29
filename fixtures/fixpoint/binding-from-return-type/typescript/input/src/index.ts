export class Store {
  upsert(): void {}
}

export function make(): Store {
  return new Store();
}

export function drive(): void {
  const store = make();
  store.upsert();
}
