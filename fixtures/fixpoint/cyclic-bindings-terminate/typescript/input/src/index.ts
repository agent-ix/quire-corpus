export class Store {
  upsert(): void {}
}

export class Cache {
  upsert(): void {}
}

export function drive(): void {
  const a = b;
  const b = c;
  const c = a;
  a.upsert();
}
