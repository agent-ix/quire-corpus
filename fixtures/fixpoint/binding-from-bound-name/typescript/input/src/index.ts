export class Store {
  upsert(): void {}
}

export function drive(first: Store): void {
  const second = first;
  const third = second;
  third.upsert();
}
